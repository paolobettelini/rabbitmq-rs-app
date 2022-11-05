const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');


module.exports = {
  entry: {
    login: "./www/login.js"
  },
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "[name].bundle.js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin({
      patterns: [
        // { from: "www", to: "www" },
        "www",
      ],
    })
  ],
  experiments: {
    asyncWebAssembly: true
  }
};

/*
module.exports = {
  entry: "./www/bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin({
      patterns: [
        // { from: "www", to: "www" },
        "www",
      ],
    })
  ],
  experiments: {
    syncWebAssembly: true
  }
};*/
