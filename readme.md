# Basic Website Generator

This is a static website generator built with Rust. It converts Markdown files from the `data/` directory into HTML files in the `webpage/` directory. It also creates an `index.html` file that lists all articles chronologically, from newest to oldest. This setup is ideal for a simple personal blog.

## Features

*   Converts Markdown to HTML.
*   Generates an index page with posts sorted by date.
*   Supports a common HTML template for all pages.
*   **Substack Integration**: Allows linking to a Substack post directly from your markdown.
*   **Cover Images**: Automatically embeds a cover image in posts if a corresponding image file is found.
*   **Special Page Handling**: `about.md`, `newsletter.md`, and `404.md` are generated but not listed on the index. `example.md` is excluded from production builds.
*   **Responsive Design**: Links to separate CSS files (`style.css`, `mobile.css`) for different screen sizes.
*   **SEO Friendly**: Generates thumbnail meta tags for blog posts, improving social media sharing.
*   **Navigation**: Includes a standard header with links to Home, Newsletter, and About pages. Active page links for "About" and "Newsletter" are highlighted.

## Usage

To use this generator, you need Rust installed.

1.  Clone this repository.
2.  Create a `.env` file in the root of the project (see Configuration section below).
3.  Create your articles in Markdown format in the `data/` directory.
    *   **Title**: The first line of your markdown should be the post title (e.g., `## My Awesome Post`).
    *   **Date**: The second line should be the date in `MM-DD-YYYY` format (e.g., `01-15-2024`).
    *   **Substack Link (Optional)**: The third line can be a link to your Substack post, formatted as `[substack post](your-substack-url)`.
    *   **Cover Image (Optional)**: To add a cover image, place an image file (e.g., `my-post.jpg`, `my-post.png`) in the `images/` directory with the same name as your markdown file (e.g., `my-post.md`).
4.  Run `cargo run` in the root directory to build the website.
    *   The generator will continuously monitor the `data/` folder for changes and regenerate pages automatically for fast iteration.
5.  The generated website will be in the `webpage/` directory.

## Special Markdown File Handling

*   `about.md`: Generated as `webpage/about.html`. Linked in the header of all pages. Not included in the index page list.
*   `newsletter.md`: Generated as `webpage/newsletter.html`. Linked in the header of all pages. Not included in the index page list.
*   `404.md`: Generated as `webpage/404.html`. Not included in the index page list.
*   `example.md`: This file is intended for testing or demonstration. It will be processed during development (when running `cargo run` without the `--prod` flag) but will be skipped in production builds.

## Configuration

The project uses a `.env` file in the root directory for configuration:

*   `BASE_URL`: The base URL of your website (e.g., `https://example.com`).
*   `TITLE`: The title of your website (e.g., `My Personal Blog`).

Example `.env` file:
```
BASE_URL=https://yourdomain.com
TITLE=My Awesome Blog
```

**CLI Flag: `--prod`**

*   By default (`cargo run`), the `BASE_URL` is adjusted for local development, pointing to the local `webpage/` directory.
*   To build for production using the `BASE_URL` from your `.env` file, run: `cargo run -- --prod`.

## License

This project is licensed under the MIT License. See the `LICENSE` file for more details.
