/** @type {import('tailwindcss').Config} */
module.exports = {
    mode: "all",
    content: [
        // include all rust, html and css files in the src directory
        "./src/**/*.{rs,html,css}",
        // include all html files in the output (dist) directory
        "./dist/**/*.html",
    ],
    safelist: [
        {pattern: /text-ctp-./},
    ],
    theme: {
        extend: {},
    },
    plugins: [require("@catppuccin/tailwindcss")({
        // prefix to use, e.g. `text-pink` becomes `text-ctp-pink`.
        // default is `false`, which means no prefix
        prefix: "ctp",
        // which flavour of colours to use by default, in the `:root`
        defaultFlavour: "latte",
    }),
    ],
}

