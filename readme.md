# Basic website
This is a basic static website generator that I created for my own personal use. It converts .md files written in `/data` into .html files in `/webpages`. It also creates an index.html file in the root directory that lists all the articles from the most recent to the eldest chronological order. These can then be hosted somewhere for a simple personal blog.

It is developed using Rust, for no reason other than that's what I wanted to use.

## Usage
Usage requires Rust to be installed.

To build a website first:
1. Clone this repository
2. Fill out your .env file with the desired base url and website title
3. Create articles in markdown format in `/data`
4. Run `cargo run` in the root directory
5. The generated website will be in `/webpages`

Currently it is configured to continuously run and listen to your `/data` folder for any updates to then regenerate all the pages. That allows for fast iteration on pages.

There is special handling with the `about.md` file in the `/data` folder. This will not be indexed on the main page. Instead it will be generated as normal, but linked to on every page in the header.

## Configuration
The project relies on a .env file with two variables:
* `base_url` - the base url of the website, e.g. `https://example.com`
* `title` - title of the website, e.g. `My Blog`

Furthermore, there is one cli flag: `prod`. If you without this, then the base_url will be set to the local directory that you are developing in, e.g. `/user/github/website/webpage`. However, if you use the `prod` flag, then the base_url will be set to the value in the .env file. This is useful for testing the website locally before deploying it.

Here is an example command to run with the `prod` flag: `cargo r -- --prod`

## License
This project has an MIT license. See the LICENSE file for more details.
