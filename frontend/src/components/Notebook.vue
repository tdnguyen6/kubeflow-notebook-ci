<template>
  <div>
    <div class="block">
      <div class="block__head">
        <div class="block__title">{{ nb.name }}</div>
        <div class="block__util">
          <div v-if="nb.enabled" class="tooltip enabled">
            <fa icon="check-circle" />
            <div class="tooltip__text">Enabled</div>
          </div>
          <div v-else class="tooltip disabled">
            <fa icon="minus-circle" />
            <div class="tooltip__text">Disabled</div>
          </div>
          <div class="cta" @click="body = 'config'">
            <div class="tooltip">
              <fa icon="cog" />
              <div class="tooltip__text">Settings</div>
            </div>
          </div>
          <div class="cta" @click="body = 'build-output'">
            <div class="tooltip">
              <fa icon="eye" />
              <div class="tooltip__text">Build ouput</div>
            </div>
          </div>
          <div class="tooltip" v-if="nb.building">
            <div class="building">
              <div class="building__inner"></div>
            </div>
            <div class="tooltip__text">Currently building</div>
          </div>
          <!-- v-if="nb.enabled && !nb.building && nb.out_of_sync" -->
          <transition name="fade">
            <div v-if="nb.enabled && !nb.building && nb.out_of_sync">
              <transition name="fade">
                <div class="tooltip" v-if="nb.syncing">
                  <div class="building building--sync">
                    <div class="building__inner building--sync__inner"></div>
                  </div>
                  <div class="tooltip__text">Currently building</div>
                </div>

                <div
                  v-else
                  class="cta sync"
                  @click="$emit('syncImage', nb.name)"
                >
                  <div class="tooltip">
                    <fa icon="sync-alt" />
                    <div class="tooltip__text">Sync image</div>
                  </div>
                </div>
              </transition>
            </div>
          </transition>
        </div>
      </div>
      <div class="block__body">
        <transition name="fade">
          <div v-if="body === 'build-output'">
            <div class="close-btn" @click="body = ''">
              <fa icon="times-circle" />
            </div>
            <output-body :nb="nb" :appData="appData" />
          </div>
          <div v-else-if="body === 'config'">
            <div class="close-btn" @click="body = ''">
              <fa icon="times-circle" />
            </div>
            <config-body :nb="nb" :appData="appData" />
          </div>
        </transition>
      </div>
    </div>
  </div>
</template>

<script>
import OutputBody from "./body/OutputBody.vue";
import ConfigBody from "./body/ConfigBody.vue";
export default {
  name: "App",
  components: { OutputBody, ConfigBody },
  props: {
    nb: Object,
    appData: {},
  },
  data() {
    return {
      body: "",
    };
  },
  computed: {},
};
</script>

<style lang="scss" scoped>
.block {
  margin: 1rem 0;

  &__head {
    border: 1px solid $gray-3;
    border-radius: 0.5rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: $gray-1;
  }

  &__title {
    font-size: 1.25rem;
    font-weight: bold;
    padding: 1rem;
    text-align: left;
    flex-grow: 2;
  }

  &__body {
    border: 1px solid $gray-3;
    border-bottom-left-radius: 0.5rem;
    border-bottom-right-radius: 0.5rem;
    border-top: none;
    margin-top: -0.4rem;
    padding-top: 1rem;

    .close-btn {
      cursor: pointer;
      width: fit-content;
      font-size: 1.5rem;
      margin: 1rem 2rem 1rem auto;
      color: $gray-3;

      &:hover {
        color: black;
      }
    }
  }

  &__util {
    padding: 1rem;
    font-size: 1.5rem;
    display: flex;
    justify-content: flex-end;
    flex-wrap: wrap;
    align-items: center;
    flex-grow: 1;
    gap: 1rem;

    .cta {
      color: $gray-3;
      border-radius: 0.5rem;
      cursor: pointer;
      border: 1px solid $gray-2;
      background: white;

      &:hover {
        color: $gray-1;
        background: $gray-3;
      }
    }

    .tooltip {
      padding: 0.25rem 0.5rem;
      position: relative;

      &__text {
        position: absolute;
        bottom: 75%;
        left: 50%;
        opacity: 0.9;
        font-size: small;
        background: $gray-3;
        color: $gray-1;
        border-radius: 0.5rem;
        z-index: 1;
        padding: 0.25rem 1rem;
      }
    }

    .tooltip .tooltip__text {
      visibility: hidden;
    }

    .tooltip:hover .tooltip__text {
      visibility: visible;
    }

    .enabled {
      color: $green-success;
    }

    .disabled {
      color: #d2ac20;
    }

    .sync {
      background: $blue-cta;
      color: $gray-1;
      font-size: 1.25rem;
      display: flex;
      align-items: center;

      &:hover {
        color: white;
        background: $blue-cta-hover;
      }
    }
  }
}

.building {
  border: 3px solid #969389; /* Light grey */
  border-top: 3px solid #caae55; /* Blue */
  border-radius: 50%;
  width: 30px;
  height: 30px;
  animation: spin 1s linear infinite;
  padding: 3px;
  margin: auto;

  &__inner {
    width: 100%;
    height: 100%;
    border-radius: 50%;
    background: #d2ac20;
  }

  &--sync {
    border-top: 3px solid #3c77ca; /* Blue */
    &__inner {
      background: #3473cc;
    }
  }
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
