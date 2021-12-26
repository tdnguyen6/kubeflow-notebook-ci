<template>
  <div class="loader"></div>
</template>

<script>
export default {};
</script>

<style lang="scss" scoped>
.loader {
  border: 2vw solid $gray-1; /* Light grey */
  border-top: 2vw solid $blue-cta; /* Blue */
  border-radius: 50%;
  width: 15vw;
  height: 15vw;
  animation: spin 2s linear infinite;
  margin: auto;
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
