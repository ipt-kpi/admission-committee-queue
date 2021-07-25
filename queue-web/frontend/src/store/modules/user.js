const state = () => ({
  admin: localStorage.getItem("admin") || false,
  username: localStorage.getItem("username"),
  token: null,
  exp: null,
  refreshSession: localStorage.getItem("refreshSession"),
  login:
    localStorage.getItem("refreshSession") != null &&
    localStorage.getItem("refreshSession") > new Date().getTime() / 1000
});

const getters = {
  isAuthenticated: state => {
    return state.login;
  },
  isAdmin: (state, getters) => {
    return getters.isAuthenticated && state.admin;
  }
};

const actions = {};

const mutations = {
  login: (state, payload) => {
    if (payload.data.role === "admin") {
      localStorage.setItem("admin", "true");
      state.admin = true;
    }
    state.token = payload.data.accessToken;
    state.exp = payload.data.exp;
    state.refreshSession = payload.data.refreshSession;
    localStorage.setItem("refreshSession", payload.data.refreshSession);
    localStorage.setItem("username", payload.login);
    state.username = payload.login;
    state.login = true;
  },
  refreshSession: (state, payload) => {
    state.token = payload.data.accessToken;
    state.exp = payload.data.exp;
    state.refreshSession = payload.data.refreshSession;
    localStorage.setItem("refreshSession", payload.data.refreshSession);
  },
  logout: state => {
    localStorage.removeItem("admin");
    state.admin = false;
    state.username = "";
    localStorage.removeItem("username");
    state.token = null;
    state.exp = null;
    state.refreshSession = null;
    localStorage.removeItem("refreshSession");
    state.login = false;
  }
};

export default {
  namespaced: true,
  state,
  getters,
  actions,
  mutations
};
