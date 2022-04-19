// Install tailwindcss standalone CLI, see https://tailwindcss.com/blog/standalone-cli.
// Or install node version of tailwindcss.
//
// tailwindcss -i zine-entry.css -o static/zine.css --watch --minify
const colors = require('tailwindcss/colors');

module.exports = {
    content: [
        './templates/**/*.jinja',
    ],
    theme: {
        extend: {
            colors: {
                primary: 'var(--primary-color)',
                main: 'var(--main-color)',
                secondary: 'var(--secondary-color)',
                link: 'var(--link-color)',
            },
            typography: {
                DEFAULT: {
                    css: {
                        a: {
                            color: 'var(--link-color)',
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
                            borderLeftWidth: '2px',
                            borderLeftColor: 'var(--primary-color)',
                            // Make font weight and style to normal
                            fontWeight: '400',
                            fontStyle: 'normal',
                            // Disable blockquote's quotes style
                            quotes: 'none',
                            paddingLeft: '0.8rem',
                        },
                        // Customize the color of strong inside blockquote
                        'blockquote strong': {
                            color: '#6c6d6d',
                        },
                        ol: {
                            paddingLeft: '1rem',
                        },
                        ul: {
                            paddingLeft: '1rem',
                        },
                        'ol > li::marker': {
                            fontWeight: '400',
                            color: 'var(--primary-color)',
                        },
                        'ul > li::marker': {
                            color: 'var(--primary-color)',
                        },
                    },
                },
                // Customize the essential prose-slate colors.
                // Mainly for article comments UI.
                slate: {
                    css: {
                        '--tw-prose-body': colors.slate[500],
                        '--tw-prose-headings': colors.slate[600],
                        '--tw-prose-lead': colors.slate[400],
                        '--tw-prose-links': colors.slate[500],
                        '--tw-prose-bold': colors.slate[600],
                        '--tw-prose-quotes': colors.slate[400],
                    }
                }
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