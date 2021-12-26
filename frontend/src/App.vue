<template>
  <div>
    <modal :config="modalConfig" @closeModal="closeModal" />
    <h1>Your Notebooks</h1>
    <div class="nb-utils">
      <div class="nb-utils__cta search" @click.self="showSearch = !showSearch">
        <div class="search-icon" @click="showSearch = !showSearch">
          <fa icon="search" />
        </div>
        <transition name="fade">
          <div v-show="showSearch" class="search-box">
            <input
              v-model="searchKey"
              type="text"
              placeholder="search keywords"
            />
          </div>
        </transition>
      </div>
      <div
        class="nb-utils__cta"
        :class="{ active: sortDir === 1 }"
        @click="sortDir = 1"
      >
        <fa icon="sort-alpha-up" />
      </div>
      <div
        class="nb-utils__cta"
        :class="{ active: sortDir === -1 }"
        @click="sortDir = -1"
      >
        <fa icon="sort-alpha-down-alt" />
      </div>
      <div class="tooltip">
        <div class="nb-utils__cta sync" @click="showSearchKey()">
          <fa icon="sync-alt" />
        </div>
        <div class="tooltip__text">Sync images of all notebooks</div>
      </div>
    </div>
    <div class="nb-list">
      <transition-group name="fade">
        <notebook
          v-for="(nb, i) in notebooksView"
          :key="i"
          :nb="nb"
          :appData="appData"
        />
      </transition-group>
    </div>
  </div>
</template>

<script>
import Modal from "./components/modals/Modal.vue";
import Notebook from "./components/Notebook.vue";
export default {
  name: "App",
  components: { Notebook, Modal },
  data() {
    return {
      appData: {},
      searchKey: "",
      showSearch: false,
      sortDir: 1,
      modalConfig: {
        show: false,
        type: "",
      },
    };
  },
  created() {
    // let res = await fetch("notebooks.json");
    // this.notebooks = await res.json();
    // this.appData = {
    //   notebooks: JSON.parse(sessionStorage.notebooks),
    //   enabledNotebooks: JSON.parse(sessionStorage.enabledNotebooks),
    //   k8sSecrets: JSON.parse(sessionStorage.k8sSecrets),
    //   namespace: sessionStorage.namespace,
    // };
    // console.log('before delete');
    // sessionStorage.removeItem("notebooks");
    // sessionStorage.removeItem("enabledNotebooks");
    // sessionStorage.removeItem("k8sSecrets");
    // sessionStorage.removeItem("namespace");
    // console.log('after delete');
    this.modalConfig = {
      show: true,
      type: "loading",
    };
    this.initData().then(() => this.closeModal());
    setInterval(() => this.initData(), 1000);
  },
  computed: {
    notebooksView: {
      get() {
        return this.appData.notebooks
          ? this.appData.notebooks
              .filter((n) =>
                n.name
                  .toLowerCase()
                  .includes(this.searchKey.trim().toLowerCase())
              )
              .sort((a, b) =>
                this.sortDir === 1
                  ? this.cmpToInt(a.name, b.name)
                  : -this.cmpToInt(a.name, b.name)
              )
          : [];
      },
    },
  },
  methods: {
    cmpToInt(a, b) {
      return a > b ? 1 : -1;
    },
    showSearchKey() {
      console.log(this.searchKey);
    },
    closeModal() {
      this.modalConfig = {
        show: false,
        type: "",
      };
    },
    async initData() {
      let res = await fetch(
        `${process.env.VUE_APP_BACKEND_HOST}/api/frontend-data?email=${sessionStorage.userid}`
      );
      let j = await res.json();
      this.appData = j;
    },
  },
};
</script>

<style lang="scss">
@import url("https://fonts.googleapis.com/css2?family=Roboto:wght@300&display=swap");

body {
  margin: 0;
  padding: 0;
  *,
  *:before,
  *:after {
    box-sizing: border-box;
  }
}

#app {
  width: 90%;
  margin: 3rem auto;

  @media (min-width: 900px) {
    width: 75%;
  }

  @media (min-width: 1500px) {
    width: 50%;
  }

  font-family: Roboto, Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}

.nb-utils {
  display: flex;
  justify-content: flex-end;
  margin: 2rem;
  align-items: center;
  gap: 1rem;
  font-size: 1.25rem;
  &__cta {
    border: 1px solid $gray-2;
    border-radius: 0.5rem;
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    &:hover {
      border-color: $gray-3;
      background: $gray-2;
    }
  }

  .active {
    border-color: $gray-3;
    background: $gray-2;
  }

  .search {
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 0;

    &-icon {
      padding: 0.25rem 0.5rem;
    }

    &-box {
      input {
        font-size: 1.25rem;
        border: none;
        border-bottom: 1px solid $gray-2;
      }
    }
  }

  .sync {
    background: $blue-cta;
    color: $gray-1;
    &:hover {
      background: $blue-cta-hover;
    }
  }

  .tooltip {
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
}

// .nb-list {
//   max-height: 80vh;
//   overflow-y: scroll;
//   overflow-x: hidden;
// }

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.5s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
