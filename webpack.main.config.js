const path = require('path');
const webpack = require('webpack');

module.exports = (env, argv) => {
  const isProduction = argv.mode === 'production';

  return {
    target: 'electron-main',
    mode: isProduction ? 'production' : 'development',
    devtool: isProduction ? 'source-map' : 'eval-source-map',
    entry: {
      main: './src/main/main.ts',
      preload: './src/main/preload.ts',
    },
    output: {
      path: path.resolve(__dirname, 'dist/main'),
      filename: '[name].js',
      clean: true,
    },
    resolve: {
      extensions: ['.ts', '.js', '.json'],
      alias: {
        '@': path.resolve(__dirname, 'src'),
        '@main': path.resolve(__dirname, 'src/main'),
        '@renderer': path.resolve(__dirname, 'src/renderer'),
        '@api': path.resolve(__dirname, 'src/api'),
        '@database': path.resolve(__dirname, 'src/database'),
        '@physics': path.resolve(__dirname, 'src/physics'),
        '@editor': path.resolve(__dirname, 'src/editor'),
        '@reports': path.resolve(__dirname, 'src/reports'),
        '@types': path.resolve(__dirname, 'src/types'),
        '@utils': path.resolve(__dirname, 'src/utils'),
      },
    },
    module: {
      rules: [
        {
          test: /\.ts$/,
          exclude: /node_modules/,
          use: {
            loader: 'ts-loader',
            options: {
              transpileOnly: !isProduction,
              configFile: 'tsconfig.json',
            },
          },
        },
      ],
    },
    plugins: [
      new webpack.DefinePlugin({
        'process.env.NODE_ENV': JSON.stringify(isProduction ? 'production' : 'development'),
      }),
    ],
    externals: {
      electron: 'commonjs electron',
    },
    node: {
      __dirname: false,
      __filename: false,
    },
    optimization: {
      minimize: isProduction,
    },
    stats: {
      errorDetails: true,
    },
  };
};
