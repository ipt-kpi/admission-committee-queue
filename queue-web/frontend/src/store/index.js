import Vue from "vue";
import Vuex from "vuex";
import user from "../store/modules/user";
import message from "../store/modules/message";

Vue.use(Vuex);

export default new Vuex.Store({
  state: {
    text: "",
    error: false,
    ok: false
  },
  mutations: {
    error: (state, payload) => {
      state.error = true;
      state.text = payload;
    },
    ok: (state, payload) => {
      state.ok = true;
      state.text = payload;
    },
    close: state => {
      state.error = false;
      state.ok = false;
      state.text = "";
    }
  },
  getters: {
    hasMessage: state => {
      return state.ok || state.error;
    },
    messageColor: state => {
      if (state.error) {
        return "red darken-1";
      } else if (state.ok) {
        return "green darken-1";
      }
      return null;
    }
  },
  modules: {
    user,
    message
  }
});
