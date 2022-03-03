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
      typography: {
        DEFAULT: {
          css: {
            a: {
              color: 'var(--primary-color)',
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
    require('@tailwindcss/typography'),
  ],
}