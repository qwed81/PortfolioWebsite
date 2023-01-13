const path = require('path');
const productionMode = false;
const HtmlWebpackPlugin = require('html-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');

module.exports = {
	mode: productionMode ? 'production' : 'development', 
	entry: './src/index.tsx',
	module: {
		rules: [{
			test: /\.tsx$/,
			use: 'ts-loader',
			exclude: /node_modules/,
			resolve: {
				extensions: ['.tsx', '.ts', '.js'],
			}
		},
		{
			test: /\.css$/,
			use: [MiniCssExtractPlugin.loader, 'css-loader']
		}]
	},
	output: {
		filename: 'bundle.js',
		path: path.resolve(__dirname, 'dist'),
	},
	devServer: {
		static: {
			directory: path.join(__dirname, 'dist'),
		},
		compress: true,
		port: 8080,
	},
	devtool: productionMode ? undefined : 'source-map',
	plugins: [
		new HtmlWebpackPlugin({
			template: 'src/index.html'
		}),
		new MiniCssExtractPlugin(),
	]
}
