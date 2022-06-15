# WeatherBot

This is just a simple bot that I decided to make for fun, grabs weather information using a slash command.
Unfortunately I cannot host this for other people since the API I am using is limited to 60 requests a minute, which could get quickly overloaded.

## Installation

### Requires

1. `rustup` toolchain
2. An internet connection

### Instructions

1. Run `git clone` on this repository to get a local copy
2. Build a release version of this program by running `cargo build --release`
3. Put the `.env.example` into a directory with your exectuable, and rename it to `.env`.
4. Replace `DISCORD_TOKEN` with your discord token and `API_KEY` with an API key from [https://openweathermap.org/](https://openweathermap.org/)
5. Run the executable and pray that my code is not horrendous