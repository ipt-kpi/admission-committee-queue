const state = () => ({
  text: "",
  error: false,
  ok: false
});

const getters = {
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
};

const actions = {};

const mutations = {
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
};

export default {
  namespaced: true,
  state,
  getters,
  actions,
  mutations
};
