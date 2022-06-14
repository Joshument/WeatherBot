use std::env;

use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::interactions::application_command::{ApplicationCommand, ApplicationCommandOptionType};
use serenity::model::prelude::*;
use serenity::prelude::*;
use crate::application_command::ApplicationCommandInteractionDataOptionValue;
use reqwest;
use serde::Deserialize;
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

#[derive(Deserialize)]
struct ZeroGeocode {
    lat: f32,
    lon: f32,
}

#[derive(Deserialize)]
struct WeatherResponse {
    current: Current,
}

#[derive(Deserialize)]
struct GeocodeResponse {
    zero: ZeroGeocode
}
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let api_key = env::var("API_KEY").expect("Failed to load API_KEY variable!");

        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "weather" => {
                    let city = if let ApplicationCommandInteractionDataOptionValue::String(num) 
                        = command.data.options.get(0).expect("did not get parameter").resolved.as_ref().expect("idk") {num} else {"idk"};

                    // let latitude: f32 = command.data.options.get(0).expect("didn't get the parameter!").resolved.as_ref().expect("idk");
                    let state = if let ApplicationCommandInteractionDataOptionValue::String(num) 
                        = command.data.options.get(1).expect("did not get parameter").resolved.as_ref().expect("idk") {num} else {"idk"};

                    let country = if let ApplicationCommandInteractionDataOptionValue::String(num) 
                        = command.data.options.get(2).expect("did not get parameter").resolved.as_ref().expect("idk") {num} else {"idk"};

                    let geocode_response = match reqwest::get(
                        format!(
                            "http://api.openweathermap.org/geo/1.0/direct?q={},{},{}&limit=1&appid={}",
                            city,
                            state,
                            country,
                            api_key
                        )
                    ).await.expect("Failed to get API response!").json::<GeocodeResponse>().await {
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
                            "https://api.openweathermap.org/data/2.5/onecall?lat={}&lon={}&appid={}&exclude=minutely,hourly,daily,alerts", 
                            geocode_response.zero.lat, 
                            geocode_response.zero.lon, 
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
                                        embed.title(format!("Weather for {}, {}", city, state))
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
                command.name("sendembed").description("send a test embed")
            }).await
        );

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
                            .name("state")
                            .description("State or Province of the location")
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
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}