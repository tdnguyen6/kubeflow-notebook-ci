<template>
  <div class="main">
    <modal :config="modalConfig" @closeModal="closeModal" />
    <div class="form">
      <div class="form__title">Notebook CI Settings</div>
      <div class="form__section">
        <div class="section section--horizontal">
          <div class="section__title">Enable</div>
          <div class="section__body">
            <toggle-switch
              @click="form.enable = !form.enable"
              :value="form.enable"
            />
          </div>
        </div>
      </div>
      <transition name="fade">
        <div class="form__section" v-show="form.enable">
          <div class="section">
            <div class="section__title">Repository</div>
            <div class="section__body">
              <div class="block">
                <div class="block__title">
                  URI
                  <div
                    class="block__explain-toggle"
                    :class="{
                      'block__explain-toggle--active': explainations.repo.uri,
                    }"
                    @click="explainations.repo.uri = !explainations.repo.uri"
                  >
                    <fa icon="question-circle" />
                  </div>
                </div>
                <transition name="fade">
                  <div class="block__explain" v-if="explainations.repo.uri">
                    <table>
                      <tr>
                        <th>Format:</th>
                        <td>
                          [protocol://][repository](//optional-pathspec)(@optional-reference)
                        </td>
                      </tr>
                      <tr>
                        <th>Supported Protocols</th>
                        <td>http | https | ssh</td>
                      </tr>
                      <tr>
                        <th>Path Spec</th>
                        <td>Sub path inside repository</td>
                      </tr>
                      <tr>
                        <th>Git Reference</th>
                        <td>
                          Anything that can be checked out, including:
                          <ul>
                            <li>Branches</li>
                            <li>Tags</li>
                            <li>Commits</li>
                          </ul>
                        </td>
                      </tr>
                      <tr>
                        <th>Valid URIs:</th>
                        <td>
                          https://github.com/user/project.git//foo@c7aee6bd
                          <br />
                          http://github.com/acme/myproject.git//bar@branches/main
                          <br />
                          https://bitbucket.org/user/project.git//baz@tags/my-tag
                          <br />
                          ssh://github.com/acme/myproject.git//baz
                          <br />
                          ssh://github.com/acme/myproject@tags/my-tag
                          <br />
                          ssh://github.com/acme/myproject@branches/my-branch
                        </td>
                      </tr>
                    </table>
                  </div>
                </transition>
                <div class="block__input">
                  <input
                    type="text"
                    v-model="form.repo.uri"
                    placeholder="[protocol://][repository](//optional-pathspec)(@optional-reference)"
                  />
                </div>
              </div>
              <div class="block block--horizontal">
                <div class="block__title">Private Repository</div>
                <div class="block__input">
                  <toggle-switch
                    @click="form.repo.private = !form.repo.private"
                    :value="form.repo.private"
                  />
                </div>
              </div>
              <transition name="fade">
                <div class="block" v-if="form.repo.private">
                  <div class="block__title">
                    Git Credential Secret
                    <div
                      class="block__explain-toggle"
                      :class="{
                        'block__explain-toggle--active':
                          explainations.repo.secret,
                      }"
                      @click="
                        explainations.repo.secret = !explainations.repo.secret
                      "
                    >
                      <fa icon="question-circle" />
                    </div>
                  </div>
                  <transition name="fade">
                    <div
                      class="block__explain"
                      v-if="explainations.repo.secret"
                    >
                      Needed data keys based on protocol:
                      <table>
                        <tr>
                          <th>HTTP(S)</th>
                          <td>
                            GIT_USERNAME <br />
                            GIT_PASSWORD (this can also be an access token)
                          </td>
                        </tr>
                        <tr>
                          <th>SSH</th>
                          <td>GIT_SSH_KEY</td>
                        </tr>
                      </table>
                    </div>
                  </transition>
                  <div class="block__input">
                    <input
                      type="text"
                      v-model="form.repo.secret"
                      list="k8sSecrets"
                    />
                    <datalist id="k8sSecrets">
                      <option v-for="(s, i) in k8sSecrets" :key="i" :value="s">
                        {{ s }}
                      </option>
                    </datalist>
                  </div>
                </div>
              </transition>
            </div>
          </div>
        </div>
      </transition>
      <transition name="fade">
        <div class="form__section" v-show="form.enable">
          <div class="section">
            <div class="section__title">Container Image</div>
            <div class="section__body">
              <div class="block">
                <div class="block__title">Name</div>
                <!-- <div class="block__explain"></div> -->
                <div class="block__input">
                  <input
                    type="text"
                    v-model="form.image.name"
                    placeholder="registry/name:tag"
                  />
                </div>
              </div>
              <div class="block block--horizontal">
                <div class="block__title">Private Registry</div>
                <div class="block__input">
                  <toggle-switch
                    @click="form.image.private = !form.image.private"
                    :value="form.image.private"
                  />
                </div>
              </div>
              <transition name="fade">
                <div class="block" v-if="form.image.private">
                  <div class="block__title">
                    Private registry type
                    <div
                      class="block__explain-toggle"
                      :class="{
                        'block__explain-toggle--active':
                          explainations.image.registry,
                      }"
                      @click="
                        explainations.image.registry =
                          !explainations.image.registry
                      "
                    >
                      <fa icon="question-circle" />
                    </div>
                  </div>
                  <div
                    class="block__explain"
                    v-if="explainations.image.registry"
                  >
                    Supports basic authentication only
                  </div>
                  <div class="block__input">
                    <input
                      type="text"
                      v-model="form.image.registry"
                      :placeholder="conventionalRegistry()"
                    />
                  </div>
                </div>
              </transition>
              <transition name="fade">
                <div class="block" v-if="form.image.private">
                  <div class="block__title">
                    Container Registry Credential Secret
                    <div
                      class="block__explain-toggle"
                      :class="{
                        'block__explain-toggle--active':
                          explainations.image.secret,
                      }"
                      @click="
                        explainations.image.secret = !explainations.image.secret
                      "
                    >
                      <fa icon="question-circle" />
                    </div>
                  </div>
                  <div class="block__explain" v-if="explainations.image.secret">
                    Needed data keys:
                    <ul>
                      <li>CR_USERNAME</li>
                      <li>CR_PASSWORD (this can also be an access token)</li>
                    </ul>
                  </div>
                  <div class="block__input">
                    <input
                      type="text"
                      v-model="form.image.secret"
                      list="k8sSecrets"
                    />
                    <datalist id="k8sSecrets">
                      <option v-for="(s, i) in k8sSecrets" :key="i" :value="s">
                        {{ s }}
                      </option>
                    </datalist>
                  </div>
                </div>
              </transition>
            </div>
          </div>
        </div>
      </transition>
      <transition name="fade">
        <div class="form__section" v-show="form.enable">
          <div class="section section--horizontal">
            <div class="section__title">
              Auto syncing pod to image after build
            </div>
            <div class="section__body">
              <toggle-switch
                @click="form['auto-sync'] = !form['auto-sync']"
                :value="form['auto-sync']"
              />
            </div>
          </div>
        </div>
      </transition>
      <div class="form__submit">
        <button @click="submit()">Save</button>
      </div>
    </div>
  </div>
</template>

<script>
import Modal from "../modals/Modal.vue";
import ToggleSwitch from "../ToggleSwitch.vue";
export default {
  components: { ToggleSwitch, Modal },
  data() {
    return {
      form: {
        enable: this.$props.nb.enabled,
        repo: {
          id: -1,
          uri: "",
          private: false,
          secret: "",
        },
        image: {
          name: "",
          private: false,
          registry: "",
          secret: "",
        },
        "auto-sync": false,
      },
      k8sSecrets: this.$props.appData.secrets,
      modalConfig: {
        show: false,
        type: "",
      },
      explainations: {
        repo: {
          uri: false,
          secret: false,
        },
        image: {
          type: false,
          secret: false,
        },
      },
    };
  },
  props: {
    appData: {},
    nb: Object,
  },
  methods: {
    submit() {
      if (this.form.enable) {
        let validated = true;
        if (
          !/^(?<protocol>http|https|ssh):\/\/(?<repo>(?:[A-Za-z0-9]+[./\-_])*(?:[A-Za-z0-9]+))(?:(?:\/\/)?(?<pathspec>(?:[A-Za-z0-9]+[/\-_])*(?:[A-Za-z0-9]+)))?(?:@(?<ref>(?:branches\/[A-ZFa-z0-9-_/]+)|(?:tags\/[A-ZFa-z0-9-_/]+)|(?:[A-Fa-f0-9]+)))?$/.test(
            this.form.repo.uri
          )
        ) {
          validated = false;
          alert("Wrong Repo URI format");
        }

        if (
          this.form.image.name === "" ||
          !/^(?:(?=[^:/]{1,253})(?!-)[a-zA-Z0-9-]{1,63}(?<!-)(?:\.(?!-)[a-zA-Z0-9-]{1,63}(?<!-))*(?::[0-9]{1,5})?\/)?((?![._-])(?:[a-z0-9._-]*)(?<![._-])(?:\/(?![._-])[a-z0-9._-]*(?<![._-]))*)(?::(?![.-])[a-zA-Z0-9_.-]{1,128})?$/.test(
            this.form.image.name
          )
        ) {
          validated = false;
          alert("Wrong image name");
        }

        if (validated) {
          this.modalConfig = {
            show: true,
            type: "loading",
          };
          fetch(
            `${process.env.VUE_APP_BACKEND_HOST}/api/notebook?name=${this.nb.name}&namespace=${this.appData.namespace}`,
            {
              method: "PUT",
              body: JSON.stringify({
                image: this.form.image.name,
                registry_credential_secret: this.form.image.secret,
                private_registry: this.form.image.private,
                registry:
                  this.form.image.registry === ""
                    ? this.conventionalRegistry()
                    : this.form.image.registry,
                repo_id: this.form.repo.id,
                repo_uri: this.form.repo.uri,
                private_repo: this.form.repo.private,
                repo_credential_secret: this.form.repo.secret,
                auto_sync: this.form["auto-sync"],
              }),
              headers: {
                "Content-Type": "application/json",
              },
            }
          )
            .then(() => {
              this.modalConfig = {
                show: true,
                type: "success",
              };
            })
            .catch(() => {
              this.modalConfig = {
                show: true,
                type: "failure",
              };
            });
        }
      } else {
        this.modalConfig = {
          show: true,
          type: "loading",
        };
        fetch(
          `${process.env.VUE_APP_BACKEND_HOST}/api/notebook?name=${this.nb.name}&namespace=${this.appData.namespace}`,
          {
            method: "DELETE",
          }
        )
          .then()
          .catch()
          .finally(this.closeModal());
      }
    },
    closeModal() {
      this.modalConfig = {
        show: false,
        type: "",
      };
    },
    conventionalRegistry() {
      return this.form.image.name.split("/")[0];
    },
  },
  created() {
    if (this.nb.enabled) {
      this.modalConfig = {
        show: true,
        type: "loading",
      };
      fetch(
        `${process.env.VUE_APP_BACKEND_HOST}/api/notebook?name=${this.nb.name}&namespace=${this.appData.namespace}`
      )
        .then((res) => res.json())
        .then((d) => {
          this.form.repo = {
            uri: d.repo_uri,
            private: d.private_repo,
            secret: d.repo_credential_secret,
            id: d.repo_id,
          };
          (this.form.image = {
            name: d.image,
            private: d.private_registry,
            type: d.registry,
            secret: d.registry_credential_secret,
          }),
            (this.form["auto-sync"] = d.auto_sync);
        })
        .catch()
        .finally(this.closeModal());
    }
  },
};
</script>

<style lang="scss" scoped>
.main {
  text-align: left;

  .form {
    margin: 2rem;
    padding: 2rem 4rem;
    border: 1px dashed $gray-2;
    background: $gray-1;
    border-radius: 0.5rem;

    &__title {
      font-size: 1.2rem;
      text-decoration: underline;
      font-weight: bolder;
      text-align: center;
      padding: 1rem;
    }

    &__section {
      .section {
        padding: 1rem;
        margin: 1rem 0;
        border-bottom: 1px solid $gray-2;

        &--horizontal {
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        &__title {
          font-weight: bold;
          font-size: 1rem;
        }

        &__body {
          .block {
            &--horizontal {
              display: flex;
              justify-content: space-between;
              align-items: center;
              align-content: center;
            }

            margin: 2rem 0 1rem 0.5rem;
            padding-left: 1rem;
            border-left: 1px solid $gray-2;

            &__title {
              margin: 1rem 0;
              font-weight: bold;
              font-style: italic;
              font-size: 0.9rem;
              display: flex;
              align-items: center;
              justify-content: left;
              gap: 0.5rem;
            }

            &__explain-toggle {
              color: $gray-3;
              cursor: pointer;
              font-size: 1.2rem;

              &:hover,
              &--active {
                color: black;
              }
            }

            &__explain {
              font-size: small;
              margin: 1rem 0 1rem 0.5rem;
              padding: 1rem;
              background: $gray-0;
              border-left: 1px solid $gray-2;

              table {
                border-collapse: separate;
                border-spacing: 0.5rem 1.5rem;
                tr {
                  td {
                    word-wrap: break-word;
                    overflow-wrap: break-word;
                    word-break: break-word;
                  }
                }
              }
            }

            &__input {
              input,
              select {
                padding: 0.5rem;
                border-radius: 0.25rem;
                border: 1px solid $gray-2;
                width: 100%;
              }
            }
          }
        }
      }
    }

    &__submit {
      text-align: center;
      margin-top: 1rem;
      padding-top: 1rem;
      button {
        cursor: pointer;
        background: $blue-cta;
        border: 1px solid $gray-3;
        color: $gray-1;
        padding: 0.5rem 1rem;
        border-radius: 0.5rem;

        &:hover {
          background: $blue-cta-hover;
        }
      }
    }
  }
}
</style>
