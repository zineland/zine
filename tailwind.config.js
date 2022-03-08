// Install tailwindcss standalone CLI, see https://tailwindcss.com/blog/standalone-cli.
// Or install node version of tailwindcss.
//
// tailwindcss -o target/zine.css --watch --minify
module.exports = {
    content: [
        './templates/**/*.jinja',
    ],
    theme: {
        extend: {
            colors: {
                primary: 'var(--primary-color)',
                secondary: 'var(--secondary-color)',
            },
            typography: {
                DEFAULT: {
                    css: {
                        a: {
                            color: 'var(--primary-link-color)',
                            textDecoration: 'none',
                            fontWeight: '400',
                        },
                        'a:hover': {
                            textDecoration: 'underline',
                        },
                        strong: {
                            fontWeight: '500',
                        },
                        blockquote: {
                            color: "#7c8088",
                            // Make font weight and style to normal
                            fontWeight: '400',
                            fontStyle: 'normal',
                            // Disable blockquote's quotes style
                            quotes: 'none',
                        },
                    },
                },
            },
        },
    },
    plugins: [
        // A plugin to pretty markdown content.
        require('@tailwindcss/typography')({
            target: 'legacy', // disables :where() selectors
        }),
        // A plugin to truncate text to a fixed number of lines.
        require('@tailwindcss/line-clamp'),
    ],
}