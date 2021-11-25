module.exports = {
  lintOnSave: false,
  publicPath : (
    process.env.NODE_ENV === 'production'
  ) ? './' : '/',

  outputDir : 'dist',
  css: {
    extract: true,
    requireModuleExtension: true,
    loaderOptions: {
      sass: {
        prependData: `
          
        `
      }
    }
  },
  devServer: {
    port : 3200,
    https : false,
    disableHostCheck: true,
  }
}