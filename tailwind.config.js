/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*"],
  theme: {
    extend: {
      colors: {
        indigo: {
          1000: "hsl(244 47% 7% / 1)",
        },
      },
    },
  },
  plugins: [require("@tailwindcss/typography"), require("@tailwindcss/forms")],
};
