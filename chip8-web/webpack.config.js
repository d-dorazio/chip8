const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const CleanWebpackPlugin = require('clean-webpack-plugin')

const dist = path.resolve(__dirname, "dist");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
  entry: { main: "./js/index.js" },

  output: {
    path: dist,
    filename: "bundle.js",
    chunkFilename: '[name].bundle.js',
  },

  devServer: {
    contentBase: dist,
  },

  plugins: [
    new CleanWebpackPlugin(['dist']),

    new HtmlWebpackPlugin({
      template: 'index.html'
    }),

    new MiniCssExtractPlugin({
      filename: "[name].css",
    }),

    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "crate")
    }),
  ],

  module: {
    rules: [{
      test: /\.css$/,
      use: [
        "style-loader",
        MiniCssExtractPlugin.loader,
        "css-loader",
      ]
    }]
  },

  optimization: {
    splitChunks: {
      cacheGroups: {
        styles: {
          name: 'styles',
          test: /\.css$/,
          chunks: 'all',
          enforce: true
        }
      }
    }
  },
}
