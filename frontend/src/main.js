import { library } from "@fortawesome/fontawesome-svg-core";
import {
  faCheck,
  faCheckCircle,
  faCog,
  faExclamation,
  faEye,
  faMinusCircle,
  faQuestionCircle,
  faSearch,
  faSortAlphaDownAlt,
  faSortAlphaUp,
  faSyncAlt,
  faTimesCircle,
} from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome";
import { createApp } from "vue";
import App from "./App.vue";

library.add(faCog);
library.add(faEye);
library.add(faSyncAlt);
library.add(faCheckCircle);
library.add(faMinusCircle);
library.add(faTimesCircle);
library.add(faSortAlphaUp);
library.add(faSearch);
library.add(faSortAlphaDownAlt);
library.add(faCheck);
library.add(faExclamation);
library.add(faQuestionCircle);

createApp(App).component("fa", FontAwesomeIcon).mount("#app");
