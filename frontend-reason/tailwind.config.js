module.exports = {
  purge: {
    enabled: true,
    content: [
      './pages/_app.js',
      './pages/**/*.re',
      './pages/**/*.res',
      './components/**/*.re',
      './components/**/*.res',
    ],
  },
  darkMode: 'media', // or 'media' or 'class'
  theme: {
    extend: {},
  },
  variants: {
    extend: {},
  },
  plugins: [],
}
