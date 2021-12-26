<template>
  <div>
    <div class="loader">
      <div class="loader__content">
        <fa icon="check" />
      </div>
    </div>
    <div class="msg">Success</div>
  </div>
</template>

<script>
export default {};
</script>

<style lang="scss" scoped>
.msg {
  margin-top: 2vw;
  font-size: 2rem;
  text-align: center;
}

.loader {
  border: 2vw solid $green-success; /* Light grey */
  border-radius: 50%;
  width: 15vw;
  height: 15vw;
  animation: spin 0.5s linear;
  margin: auto;
  font-size: 9vw;
  color: $green-success;

  &__content {
    margin-top: 1vw;
    margin-left: 1vw;
  }
}

@keyframes spin {
  from {
    transform: rotate(0deg);
    color: transparent;
    border: 2vw solid $gray-1;
    border-left: 2vw solid $green-success; /* Blue */
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
