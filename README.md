# slim-osc

an incredibly thin program for displaying pc stats in a vrchat message using [osc](https://wiki.vrchat.com/wiki/Open_Sound_Control#Chatbox).

this project is licensed under the [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE).

## installation

1. download the latest release from the [releases page](https://github.com/jackssrt/slim-osc/releases)
2. configure according to [the configuration docs](#configuration).

## configuration

the configuration is stored in a [toml](https://toml.io/en/) file whose location is
- passed in using `--config`
- by default on windows: `%APPDATA%\slim-osc\config.toml`
- by default elsewhere: `$XDG_CONFIG_HOME/slim-osc/config.toml` (defaults to `~/.config/slim-osc/config.toml`)

the program will hot reload the configuration if it detects a change to it.

### example
```toml
# multi-line string - learn more at https://toml.io/en/
status = """\ 
{time(%H'%M) | superscript}{sep}{music(status) | lower | map(playing => :D, paused => :c, stopped => @.@)}{sep}{music(title) | marquee(10,20)} ᵇʸ {music(artist) | trunc(10)}\
{cpu_usage} ᶜᵖᵘ{sep}{gpu_usage} ᵍᵖᵘ{sep}{memory_usage} ᵐᵉᵐ\
"""

separator = " - " # default
address = "127.0.0.1" # default
port = "9000" # default
update_interval = { secs = 1, nanos = 0 } # default
music_backend = "playerctl" # default on linux, media_session is the default on windows
```

### the status dsl

any text is inserted literally into the status message:
`status = "meow"` => meow

you can interpolate dynamic values using the following syntax `{source(params) | filter | filter2 | filter3}`, some sources don't have any parameters and you can use how many filters you like.  
PS. dsl = domain specific language

#### some examples

`status = "{music(title)}"` => `Never Gonna Give You Up`  
`status = "{music(title) | lowercase}"` => `never gonna give you up`  
`status = "{music(title) | lowercase | superscript}"` => `ⁿᵉᵛᵉʳ ᵍᵒⁿⁿᵃ ᵍᶦᵛᵉ ʸᵒᵘ ᵘᵖ`  

### sources

#### separator / sep

evaluates to whatever the separator specified in the config file is.
`status = "{sep}"` => ` - `  

#### datetime / time / date

takes in a format passed to `chrono` which supports [these escape sequences](https://docs.rs/chrono/latest/chrono/format/strftime/index.html).

`status = "{time(%H:%S)}` => `04:02` (please go to bed)  
`status = "{date(%Y.%m.%d)}` => `2026.06.01`  

#### command / cmd

basic command source. available everywhere. outputs stdout of the command you pass in.

`status = "{command(whoami)}` => `jack`  

#### music

takes in a metadata / state field, what these are can change between music backends. status is a special case that must output something, and you should probably [map](#map) it to a nice emoji.
some common ones: status, title, artist

`status = "{music(title)}"` => `Never Gonna Give You Up`  
`status = "{music(artist)}"` => `Rick Astely`  
`status = "{music(status)}"` => `Playing` (with playerctl)  
`status = "{music(status)}"` => `Paused` (with playerctl)  
`status = "{music(status)}"` => `play` (with mpd)  
`status = "{music(status)}"` => `pause` (with mpd)  

#### cpu_model

outputs the model of your cpu.

`status = "{cpu_model}"` => `AMD Ryzen™ 7 5800X3D`  

#### gpu_model

outputs the model of your gpu.

`status = "{gpu_model}"` => `AMD Radeon™ RX 9070 XT`  

#### cpu_usage

outputs the usage percentage of your cpu.

`status = "{cpu_usage}"` => `90%`  

#### gpu_usage

outputs the usage percentage of your gpu.

`status = "{gpu_usage}"` => `100%`  

#### memory_usage

outputs the ratio between allocated and available memory as a percentage.
fun fact: your available memory is still being used by your operating system to cache files. if you ever see that you don't have any "free" memory, it's normal. [read more](https://linuxatemyram.com)

`status = "{gpu_usage}"` => `50%`  

#### text

discouraged, perform whatever filters you have manually and don't use an
interpolation. added because i know people are going to do it anyways and spawning echo in a shell is even worse than just adding this.

`status = "{text(test)}"` => `test`  

### filters

#### uppercase / upper

UPPERCASES whatever its given.

`status = "{whoami | upper}"` => `JACK`  

#### lowercase / lower

lowercases whatever its given.

`status = "{whoami | lower}"` => `jack`  

#### trim / strip

remove extra whitespace (spaces, tabs whatnot) around whatever its given.

`status = "{text "   some spaces around me   " | trim}"` => `some spaces around me`  

#### superscript / sup

converts whatever it's passed in to ˢᵘᵖᵉʳˢᶜʳᶦᵖᵗ. basically just tiny letters above normal ones. not all letters or symbols have superscript variants. unsupported characters (letters and symbols) are left untouched.

`status = "{cmd(whoami) | superscript}"` => `ʲᵃᶜᵏ`  

#### subscript / sub

tries its best to convert whatever it's passed in to ₛᵤbₛcᵣᵢₚₜ. basically just tiny letters below normal ones. not all letters or symbols have superscript variants. unsupported characters (letters and symbols) are left untouched.

`status = "{cmd(whoami) | superscript}"` => `ⱼₐcₖ`  

#### marquee / scroll

scrolls whatever you pass it in, takes in a target length and a period (how long one rotation should take).

`status = "{music(title) | marquee(8, 10)}"` => `Never go` (will take 10 updates to scroll the full title across 4 characters)  

#### truncate / trunc / trun

cuts off whatever is passed in after a target length, if its cut off replace the last character with an ellipsis.

`status = "{music(title) | trunc(8)}"` => `Never g…`  

#### map / m

maps the text its passed in using one or more rules defined like:

`status = "{music(status) | m(Playing => ▶️, Paused => ⏸️)}"` => `▶️`
