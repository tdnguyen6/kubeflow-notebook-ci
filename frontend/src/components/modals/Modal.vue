<template>
  <transition name="fade">
    <div class="modal" v-if="config.show">
      <div class="modal__main">
        <div class="modal__overlay" @click.self="this.$emit('closeModal')">
          <div class="modal__content">
            <loading-modal v-if="config.type === 'loading'" />
            <success-modal v-else-if="config.type === 'success'" />
            <failure-modal v-else-if="config.type === 'failure'" />
            <div v-else>{{ config.type }}</div>
          </div>
        </div>
      </div>
    </div>
  </transition>
</template>

<script>
import FailureModal from "./FailureModal.vue";
import LoadingModal from "./LoadingModal.vue";
import SuccessModal from "./SuccessModal.vue";

export default {
  components: {
    LoadingModal,
    FailureModal,
    SuccessModal,
  },
  data() {
    return {};
  },
  props: {
    config: Object,
  },
};
</script>

<style lang="scss" scoped>
.modal {
  position: fixed;
  height: 100vh;
  width: 100vw;
  z-index: 99;

  // &__main {
  // }

  &__overlay {
    height: 100vh;
    width: 100vw;
    position: fixed;
    top: 0;
    left: 0;
    background: $gray-2;
    opacity: 0.8;
    display: grid;
    justify-content: center;
    align-items: center;
  }

  &__content {
    opacity: 1;
  }
}
</style>
