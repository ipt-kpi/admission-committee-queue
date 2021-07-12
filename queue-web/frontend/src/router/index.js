import Vue from "vue";
import VueRouter from "vue-router";
import Home from "../views/Home.vue";
import Register from "@/views/Register";
import Login from "@/views/Login";
import store from "../store";

const originalPush = VueRouter.prototype.push;
VueRouter.prototype.push = function push(location) {
  return originalPush.call(this, location).catch(err => err);
};

Vue.use(VueRouter);

const ifAuthenticated = (to, from, next) => {
  if (store.getters["user/isAuthenticated"]) {
    next();
    return;
  }
  next("/login");
};

const ifNotAuthenticated = (to, from, next) => {
  if (!store.getters["user/isAuthenticated"]) {
    next();
    return;
  }
  next("/login");
};

const ifAdmin = (to, from, next) => {
  if (store.getters["user/isAdmin"]) {
    next();
    return;
  }
  next("/");
};

const routes = [
  {
    path: "/",
    name: "Home",
    component: Home
  },
  {
    path: "/register",
    name: "Register",
    component: Register,
    beforeEnter: ifNotAuthenticated
  },
  {
    path: "/login",
    name: "Login",
    component: Login,
    beforeEnter: ifNotAuthenticated
  },
  { path: "/user", beforeEnter: ifAuthenticated },
  {
    path: "/admin/queue/register",
    beforeEnter: ifAdmin
  },
  {
    path: "/admin/queue/edit",
    beforeEnter: ifAdmin
  }
];

const router = new VueRouter({
  mode: "history",
  base: process.env.BASE_URL,
  routes
});

export default router;
