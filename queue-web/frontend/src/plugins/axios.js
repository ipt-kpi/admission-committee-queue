"use strict";

import Vue from "vue";
import axios from "axios";
import store from "../store";
import fingerprint2 from "fingerprintjs2";

let config = {
  baseURL: process.env.VUE_APP_API_URL || "",
  withCredentials: true
};

const _axios = axios.create(config);

_axios.interceptors.request.use(
  async function(config) {
    let user = store.state.user;
    if (
      config.url !== "/user/auth/refresh-session" &&
      user.login &&
      (user.exp - 10 <= new Date().getTime() / 1000 || user.token == null)
    ) {
      const components = await fingerprint2.getPromise();
      const key = components
        .map(pair => {
          return pair.value;
        })
        .join();
      let response = await axios.post(
        config.baseURL + "/user/auth/refresh-session",
        {
          fingerprint: fingerprint2.x64hash128(key, 31)
        },
        { withCredentials: true }
      );
      store.commit("user/refreshSession", { data: response.data });
    }
    if (user.token) {
      config.headers.Authorization = "Bearer " + store.state.user.token;
    }
    return config;
  },
  function(error) {
    return Promise.reject(error);
  }
);

_axios.interceptors.response.use(
  function(response) {
    return response;
  },
  function(error) {
    if (error.response) {
      if (error.response.status === 401) {
        store.commit("user/logout");
      }
    } else {
      store.commit("error", "Произошла ошибка подключения");
    }
    return Promise.reject(error);
  }
);

Plugin.install = function(Vue) {
  Vue.axios = _axios;
  window.axios = _axios;
  Object.defineProperties(Vue.prototype, {
    axios: {
      get() {
        return _axios;
      }
    },
    $axios: {
      get() {
        return _axios;
      }
    }
  });
};

Vue.use(Plugin);

export default Plugin;
