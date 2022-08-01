# A simple web formular using Rust, Rocket and Bootstrap

This project shows how to create a Multi Page Application (MPA) using

- [Rust](https://www.rust-lang.org/) a programming language empowering everyone to build reliable and efficient software
- [Rocket](https://rocket.rs/) a web framework for Rust
- [Bootstrap](https://getbootstrap.com/) a powerful, extensible, and feature-packed frontend toolkit
- [Tera](https://tera.netlify.app/) a powerful, easy to use template engine for Rust (Inspired by Jinja2 and Django templates)
- [Fluent](https://github.com/projectfluent/fluent-rs) a localization system for natural-sounding translations

The result is a modern, responsive frontend that supports multiple languages (i18n) and was built using the extremely fast and memory-safe Rust framework Rocket.

## Motivation
I've been using Rust+Rocket for REST APIs for a long time, and it works very well.  
When I tried to create the first "visual" application, I couldn't find a single example of how to integrate all these libraries and frameworks together.

## Important notes
- This example is using the release candidate of rocket 0.5, unfortunately the API is not guaranteed to be stable!  
- This example is **not** tested on Windows.

## Getting started
First, this is **not** a template project. Please do not clone or fork it! 
You should download the files from the release page and copy them into **your** git repository!

### Project structure
| Directory   | Description                                                                |
|-------------|----------------------------------------------------------------------------|
| locales/    | Contains Fluent internationalization (i18n) files                          |
| npm/        | Contains a package.json file for npm used to download and update Bootstrap |
| src/        | Rust source code files                                                     |
| templates/  | Contains HTML files which use the Tera template language                   |
| deployment/ | Contains systemd and nginx configuration examples                          |


### Downloading Bootstrap
Before you can run the application you must download the Bootstrap files.  
These files are not included in this project and are not available as rust libraries.  
Since Bootstrap is officially released only on their website and via npm, these libraries would be deprecated very soon, so you need to install "npm" for this step.  
**Note:** If you use Windows tools like "cd", "mv" and "rm" must be installed!
```bash
cd npm/
npm run-script build
```

This will download the Bootstrap (and supporting) libs and place them in the *static/* directory.

The files are linked to the project only in the template *base.html.tera*.  
If you want to update or add additional libraries, you just need to modify the *npm/package.json* file and update the links in the HTML/Template files.

### Running the application
After the *static/* files are populated, all you have to do is create and run the application:
```bash
cargo build
cargo run
```

### Code structure

#### main.rs
At the top of main you can find some structs used to receive and respond data.  
Please note that these structs do not need to be json serializable by default.

This is followed by three functions representing the endpoints:
**GET  /**  
This function redirects the client to the supported language page.  

**GET  /< lang>/formular**  
This function returns the default/home page.
First, a new captcha is created and added to the shared captcha list. Have a look at the *"captchas: &State<CaptchaList>"* parameter if you want to know how to pass shared data between functions (rocket routes).  
Afterwards, the language is detected and a new *tera::context::Context* is created.  
The context variable is used to pass data from Rust to the template engine in a key=value format.  
Finally, the site is rendered and returned to the client by a *TemplateRedirect*.  
Please note that an MPA works by redirecting from one endpoint/route to another.

**POST /< lang>/formular**  
This functions processes the entered formular data.  
The html template *home.html.tera* contains a *form method="post"* element which redirects the POST request after pressing the button to this route.
The data is passed via *"task: Form<KitRequest>"* parameter and available as struct.  
The captcha is checked using the shared list we used earlier.  
Finally, the input is validated, saved to the disk in json format and a message is returned.    
The *TemplateRedirect::Flash* mechanism is used to display messages to the client.  
It is a very simple mechanic that is based on the *"if flash_type"* template code in the *home.html.tera* file and uses cookies.  
The message translation is done by using the *LOCALES.lookup(...)* function. 

**fn main()**  
The main functions starts by checking the ROCKET_ENV environment variable.    
If it is not set the application sets some default variables for testing.  
You can change/set them like this:
```text
ROCKET_ENV=production PORT=2222 RUST_APP_LOG="info" DATA_STORAGE_DIR=/tmp/ cargo run
```
Using environment variables is best practice because you can simply switch between development and production configs.

The "*rocket::Config::figment()*" code is used to set the configuration for Rocket.  
The helper functions *parse_level/port()* convert the environment variables to config strings.  

A concurrency safe list/map is created using "*let captcha_list = Arc::new(Mutex::new(HashMap::new()));*" which is used to store the captcha between requests.

Finally, everything is put together using "*rocket::custom(...)...*":  
The *.mount(...)* code sets up all routes  
The *.manage(captchas)* code makes the shared captcha list available to all functions  
The *.attach(...)* code attaches the Tera and Fluent engines to Rocket  

If the environment is set to "development" Rocket serves the *static/* directory to the client.    
This is needed to make the Bootstrap assets available, otherwise the UI will be malformed.  
In productive environment it is highly recommended to serve the *static/* directory using nginx or apache.    
They have better caching, compression and configuration options for static files then Rocket!  

Technically it is possible to include all files from the *static/* directory in the application executable.    
This can be achieved using the *include_bytes!* macro but you will need to set up a route for every single file.

Since version 0.5 rocket supports full asynchronous request handling (with an incredible performance) so it returns a *Future* on launch. 

#### captchas.rs
This module contains all functions to store and validate the captchas.  
The concept is simple, a captcha object contains a unique hash, the valid code and the creation timestamp.    
Each time a function is called the entire list/map is checked for outdated captchas which are removed.  
If a captcha is not expired and the code matches it is successfully resolved. 
Please note that his module is not considered to be secure or performant! (although it might be) 

#### lang.rs
This module contains some setup code for Fluent.  
More interesting is the function "*impl<'a> FromRequest<'a> for UserAgent*" which is used by the root route (/).  
It is a pre-route call that parses the "*accept-language*" header field set by the browser to determine the preferred language.    
The current implementation is very simple and does not implement the full specification. It would be really nice to have a crate for this ;).

#### save.rs
This module saves the request as json file in the configured directory.    
The serde lib is only included for this reason.

#### simple_captcha.rs
This module creates a simple captcha using the "captcha" crate and returns it as object with metadata information.  
Please note that the "captcha" crate is not very stable and modifying the parameters may result in a panic. 

## Internationalization in html/template code
If you look in the html.tera files you will find the Fluent code used for translation e.g: *{{ fluent(key="version", lang=lang) }}*.  
Please note that you need to pass the "lang" parameter everytime you render a template to set the correct language. 

## Deployment
If you want to deploy your application you typically need a service and proxy configuration.  
You can find example nginx and systemd config files under *deployment/*. 

## Other questions

### Can I use this to create a Single Page Application?
An SPA is based on Javascript code in the frontend calling a RESTful API in the backend.  
You can build the backend using Rust+Rocket but Rust code does not run in the frontend/browser.  
So with other words this is the wrong example.  

### Do I need to use Bootstrap?
No, you can use any Javascript/CSS framework/library you like in the frontend.  
Just add it to the npm package.json file and link it to the html/tera templates.  

## Contributing
Issues and pull requests that change:
- spelling mistakes
- update libraries
- improve the syntax/code structure 
are very welcome.

Please do not extend this example and make a pull request!  
Instead, please publish it as new project.

