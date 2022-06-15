use std::env;

use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::interactions::application_command::{ApplicationCommand, ApplicationCommandOptionType};
use serenity::model::prelude::*;
use serenity::prelude::*;
use crate::application_command::ApplicationCommandInteractionDataOptionValue;
use reqwest;
use serde::{self,Deserialize};
use dotenv;

const KELVIN_OFFSET: f32 = 273.15;

#[derive(Deserialize)]
struct Current {
    temp: f32,
    feels_like: f32,
    weather: Weather,
}

#[derive(Deserialize)]
struct Weather {
    zero: Zero,
}

#[derive(Deserialize)]
struct Zero {
    description: String,
    icon: String,
}

/*
#[derive(Deserialize)]
struct ZeroGeocode {
    lat: f32,
    lon: f32,
} */

#[derive(Deserialize)]
struct AlertInfo {
    description: String,
}

#[derive(Deserialize)]
struct WeatherResponse {
    current: Current,
    #[serde(default)]
    alerts: Option<Vec<AlertInfo>>,
}

#[derive(Deserialize, Debug)]
struct GeocodeResponse {
    lat: f32,
    lon: f32,
    name: String,
    state: String,
    country: String,
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let api_key = env::var("API_KEY").expect("Failed to load API_KEY variable!");

        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "weather" => {
                    // city should never fail as city is a required parameter
                    let city = if let ApplicationCommandInteractionDataOptionValue::String(city) = command.data.options
                        .iter().find(|x| x.name == "city").unwrap()
                        .resolved.as_ref().expect("idk") { city } else { "idk" };

                    let state = if command.data.options.len() == 2 { city } else {
                        let temp = if let ApplicationCommandInteractionDataOptionValue::String(state) = command.data.options
                            .iter().find(|x| x.name == "state").unwrap()
                            .resolved.as_ref().expect("idk") { state } else { "idk" };
                        temp
                    };

                    // let latitude: f32 = command.data.options.get(0).expect("didn't get the parameter!").resolved.as_ref().expect("idk");

                    let country = if let ApplicationCommandInteractionDataOptionValue::String(country) = command.data.options
                        .iter().find(|x| x.name == "country").unwrap()
                        .resolved.as_ref().expect("idk") { country } else { "idk" };

                    let geocode_response = match reqwest::get(
                        format!(
                            "http://api.openweathermap.org/geo/1.0/direct?q={},{},{}&limit=10&appid={}",
                            city,
                            if state == city {""} else {state},
                            country,
                            api_key
                        )
                    ).await.expect("Failed to get API response!").json::<Vec<GeocodeResponse>>().await {
                        Ok(yea) => yea,
                        Err(_) => {
                            command.create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.content("Invalid Location!")
                                    })
                            }).await.expect("Failed to send message!");
    
                            panic!("Invalid Location!")
                        }
                    };

                    println!("{:#?}", geocode_response);
                    if geocode_response.len() != 1 {
                        let mut city_options = String::new();

                        for x in geocode_response {
                            city_options.push_str(&format!("{}, {}, {}\n", x.name, x.state, x.country));
                        }

                        command.create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message.content(format!(
                                        "More than 1 city found! Please be more specific on one of the following options:\n{}",
                                        city_options
                                    ))
                                })
                        }).await.expect("Failed to send message!");

                        panic!("Ambiguous Location!")
                    }
                    
                    // This should never fail, as if it hasn't already failed there would be at least 1 element.
                    let geocode_response = geocode_response.get(0).unwrap();
                    
                    /*.unwrap_or_else(|_| {
                        command.create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message.content("Invalid Location!")
                                })
                        }).await;

                        panic!("Invalid Location!")
                    }); */

                    let api_response = reqwest::get(
                        format!(
                            "https://api.openweathermap.org/data/2.5/onecall?lat={}&lon={}&appid={}&exclude=minutely,hourly,daily", 
                            geocode_response.lat, 
                            geocode_response.lon, 
                            api_key
                        )
                    ).await.expect("Failed to get weather!").json::<WeatherResponse>().await.expect("Failed to deserialize JSON");

                    let current_celsius = api_response.current.temp - KELVIN_OFFSET;
                    let feels_like_celsius = api_response.current.feels_like - KELVIN_OFFSET;

                    let current_fahrenheit = (current_celsius * (9.0/5.0)) + 32.0; 
                    let feels_like_fahrenheit = (feels_like_celsius * (9.0/5.0)) + 32.0; 

                    let msg = command
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message.embed(|embed| {
                                        embed.title(format!(
                                            "Weather for {}, {}", 
                                            geocode_response.name,
                                            if geocode_response.state == geocode_response.name { &geocode_response.country } else { &geocode_response.state }
                                        ))
                                            .description(format!("{}", api_response.current.weather.zero.description))
                                            .image(format!("http://openweathermap.org/img/wn/{}@2x.png", api_response.current.weather.zero.icon))
                                            .fields(vec![
                                                ("Current Temperature", format!(
                                                    "{:.1}°C\n{:.1}°F", 
                                                    current_celsius,
                                                    current_fahrenheit
                                                ), true),
                                                ("Feels Like", format!(
                                                    "{:.1}°C\n{:.1}°F", 
                                                    feels_like_celsius,
                                                    feels_like_fahrenheit
                                                ), true),
                                                (
                                                    "Weather Alerts",
                                                    api_response.alerts.unwrap_or_else(|| 
                                                        vec![
                                                            AlertInfo{
                                                                description: String::from("There are no current weather alerts for this area.")
                                                            }
                                                        ]
                                                    ).get(0).unwrap().description.clone(),
                                                    false
                                                ),
                                            ])
                                    })
                                })
                        }).await;
                    
                    if let Err(why) = msg {
                        println!("Error sending message: {:?}", why);
                    }
                }
                _ => ()
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        /* let _guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        ); */

        let mut commands: Vec<Result<ApplicationCommand, _>> = Vec::new();

        commands.push(
            ApplicationCommand::create_global_application_command(&ctx.http, |command| {
                command
                    .name("weather")
                    .description("get the weather for an area")
                    .create_option(|option| {
                        option
                            .name("city")
                            .description("City of the location")
                            .kind(ApplicationCommandOptionType::String)
                            .required(true)
                    })
                    .create_option(|option| {
                        option
                            .name("country")
                            .description("Country of the location")
                            .kind(ApplicationCommandOptionType::String)
                            .required(true)
                    })
                    .create_option(|option| {
                        option
                            .name("state")
                            .description("State or Province of the location")
                            .kind(ApplicationCommandOptionType::String)
                            .required(false)
                    })
            }).await
        );

        for result in &commands {
            if let Err(why) = result {
                eprintln!("Error adding global application command: {}", why);
            }
        }
    }  
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file!");

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES;
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}