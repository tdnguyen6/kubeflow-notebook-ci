module.exports = {
  css: {
    loaderOptions: {
      sass: {
        prependData: `
            @import "@/scss/_variables.scss";
            @import "@/scss/_mixins.scss";
          `,
      },
    },
  },
  publicPath: process.env.PUBLIC_PATH,
  devServer: {
    public: process.env.BASE_URL,
  },
};
