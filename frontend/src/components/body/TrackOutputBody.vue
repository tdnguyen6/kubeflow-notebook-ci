<template>
  <div v-html="output"></div>
</template>

<script>
import Convert from "ansi-to-html";

export default {
  data() {
    return {
      output: "",
      convert: new Convert(),
      ready: false,
      updateDataInterval: null,
    };
  },
  created() {
    this.updateDataInterval = setInterval(async () => {
      if (this.nb.repo_id >= 0) {
        let res = await fetch(
          `${process.env.VUE_APP_BACKEND_HOST}/api/repo/${this.nb.repo_id}/track_log`
        );
        this.output = this.convert.toHtml(await res.text());
      }
      if (!this.ready) {
        this.$emit("ready");
        this.ready = true;
      }
    }, 1000);
  },
  beforeUnmount() {
    clearInterval(this.updateDataInterval);
  },
  props: {
    appData: {},
    nb: Object,
  },
};
</script>

<style lang="scss" scoped>
// white-space: pre-line;
// max-height: 30rem;
// overflow-y: scroll;
// text-align: left;
// background: $gray-1;
// border-radius: 0.5rem;
// margin: 2rem;
// padding: 2rem 4rem;
// border: 1px dashed $gray-2;
</style>
