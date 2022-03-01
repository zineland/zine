// Install tailwindcss standalone CLI, see https://tailwindcss.com/blog/standalone-cli.
// Or install node version of tailwindcss.
//
// tailwindcss -o target/zine.css --watch --minify
module.exports = {
    content: [
        './templates/**/*.jinja',
    ],
    theme: {
        extend: {},
    },
    plugins: [
        require('@tailwindcss/typography'),
    ],
}