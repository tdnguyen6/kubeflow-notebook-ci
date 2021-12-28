<template>
  <div class="main">
    <div class="tab-switcher">
      <div
        class="tab-switcher__tab"
        :class="{ 'tab-switcher__tab--active': tab == 'track' }"
        style="border-top-left-radius: 0.5rem;"
        @click="
          ready = false;
          tab = 'track';
        "
      >
        Track
      </div>
      <div
        class="tab-switcher__tab"
        :class="{ 'tab-switcher__tab--active': tab == 'build' }"
        @click="
          ready = false;
          tab = 'build';
        "
      >
        Build
      </div>
       <div
        class="tab-switcher__tab"
        :class="{ 'tab-switcher__tab--active': tab == 'push' }"
        style="border-top-right-radius: 0.5rem;"
        @click="
          ready = false;
          tab = 'push';
        "
      >
        Push
      </div>
    </div>
    <div class="tab-content">
      <div class="loader" v-show="!ready"></div>
      <div v-show="ready">
        <build-output-body
          :appData="appData"
          :nb="nb"
          v-if="tab == 'build'"
          @ready="ready = true"
        />
        <push-output-body
          :appData="appData"
          :nb="nb"
          v-else-if="tab == 'push'"
          @ready="ready = true"
        />
        <track-output-body
          :appData="appData"
          :nb="nb"
          v-else
          @ready="ready = true"
        />
      </div>
    </div>
  </div>
</template>

<script>
import BuildOutputBody from "./BuildOutputBody.vue";
import TrackOutputBody from "./TrackOutputBody.vue";
import PushOutputBody from "./PushOutputBody.vue";

export default {
  components: { BuildOutputBody, TrackOutputBody, PushOutputBody },
  data() {
    return {
      tab: "track",
      ready: false,
    };
  },
  props: {
    appData: {},
    nb: Object,
  },
};
</script>

<style lang="scss" scoped>
.main {
  white-space: pre-line;
  text-align: left;
  background: $gray-1;
  border-radius: 0.5rem;
  margin: 2rem;
  border: 1px dashed $gray-2;

  .tab-switcher {
    display: flex;
    justify-content: space-evenly;
    &__tab {
      width: 100%;
      text-align: center;
      padding: 1rem;
      background: $gray-2;
      cursor: pointer;
      border: 1px solid transparent;
      &:hover:not(&--active) {
        font-weight: bolder;
        border: 1px solid black;
      }
      &--active {
        background: $gray-3;
        color: $gray-1;
      }
    }
  }

  .tab-content {
    padding: 2rem;
    overflow: scroll;
    white-space: pre;
    max-height: 30rem;
    font-family: "JetBrains Mono", monospace;
    font-size: 0.9rem;
  }
}
.loader {
  border: 1vw solid $gray-2; /* Light grey */
  border-top: 1vw solid $blue-cta; /* Blue */
  border-radius: 50%;
  width: 8vw;
  height: 8vw;
  animation: spin 1s linear infinite;
  margin: 8vw auto;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
