const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: {
    login: "./www/login.js",
    register: "./www/register.js",
    upload: "./www/upload.js",
    gallery: "./www/gallery.js"
  },
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "[name].bundle.js",
  },
  mode: "production",
  plugins: [
    new CopyWebpackPlugin({
      patterns: [ "www" ],
    })
  ],
  experiments: {
    asyncWebAssembly: true
  }
};
