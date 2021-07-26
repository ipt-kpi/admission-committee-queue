<template>
  <v-container>
    <v-navigation-drawer v-model="drawer" app absolute temporary>
      <template v-slot:prepend>
        <v-list-item v-if="$store.state.user.login" two-line>
          <v-list-item-content>
            <v-list-item-title>{{
              $store.state.user.username
            }}</v-list-item-title>
          </v-list-item-content>
        </v-list-item>
      </template>
      <v-divider></v-divider>
      <v-list dense>
        <v-list-item @click="$router.push('/')" link>
          <v-list-item-action>
            <v-icon>mdi-home</v-icon>
          </v-list-item-action>
          <v-list-item-content>
            <v-list-item-title>Главная</v-list-item-title>
          </v-list-item-content>
        </v-list-item>
        <span v-if="!$store.state.user.login">
          <v-list-item @click="$router.push('/login')" link>
            <v-list-item-action>
              <v-icon>mdi-login</v-icon>
            </v-list-item-action>
            <v-list-item-content>
              <v-list-item-title>Вход</v-list-item-title>
            </v-list-item-content>
          </v-list-item>
          <v-list-item @click="$router.push('/register')" link>
            <v-list-item-action>
              <v-icon>mdi-account-box</v-icon>
            </v-list-item-action>
            <v-list-item-content>
              <v-list-item-title>Регистрация</v-list-item-title>
            </v-list-item-content>
          </v-list-item>
        </span>
        <span v-if="$store.state.user.login">
          <v-list-item @click="$router.push('/user')" link>
            <v-list-item-action>
              <v-icon>mdi-clipboard-account</v-icon>
            </v-list-item-action>
            <v-list-item-content>
              <v-list-item-title>Пользователь</v-list-item-title>
            </v-list-item-content>
          </v-list-item>
          <v-list-group
            v-if="$store.state.user.admin"
            prepend-icon="mdi-account-multiple"
            no-action
          >
            <template v-slot:activator>
              <v-list-item-title>Очередь</v-list-item-title>
            </template>
            <v-list-item link @click="$router.push('/admin/queue/register')">
              <v-list-item-action>
                <v-icon>mdi-account-multiple-plus</v-icon>
              </v-list-item-action>
              <v-list-item-content>
                <v-list-item-title>Регистрация</v-list-item-title>
              </v-list-item-content>
            </v-list-item>
            <v-list-item link @click="$router.push('/admin/queue/edit')">
              <v-list-item-action>
                <v-icon>mdi-tooltip-edit</v-icon>
              </v-list-item-action>
              <v-list-item-content>
                <v-list-item-title>Редактирование</v-list-item-title>
              </v-list-item-content>
            </v-list-item>
          </v-list-group>
          <v-list-item @click="logout" link>
            <v-list-item-action>
              <v-icon>mdi-logout</v-icon>
            </v-list-item-action>
            <v-list-item-content>
              <v-list-item-title>Выход</v-list-item-title>
            </v-list-item-content>
          </v-list-item>
        </span>
      </v-list>
    </v-navigation-drawer>
    <v-app-bar app>
      <div class="hidden-lg-and-up">
        <v-app-bar-nav-icon @click.stop="drawer = !drawer" />
      </div>
      <span id="title">
        <v-toolbar-title @click="$router.push('/')">IPT-Queue</v-toolbar-title>
      </span>
      <span class="hidden-md-and-down">
        <span v-if="$store.state.user.login">
          <v-btn class="ma-2" text to="/user">
            <span class="mr-2">Пользователь</span>
          </v-btn>
        </span>
        <span v-if="$store.state.user.admin">
          <v-menu offset-y>
            <template v-slot:activator="{ on }">
              <v-btn text v-on="on">
                Очередь
              </v-btn>
            </template>
            <v-list>
              <v-list-item @click="$router.push('/admin/queue/register')">
                <v-list-item-title>Регистрация</v-list-item-title>
              </v-list-item>
              <v-list-item @click="$router.push('/admin/queue/edit')">
                <v-list-item-title>Редактирование</v-list-item-title>
              </v-list-item>
            </v-list>
          </v-menu>
        </span>
      </span>
      <v-spacer />
      <span class="hidden-md-and-down">
        <span v-if="!$store.state.user.login">
          <v-btn
            class="ma-2"
            text
            @click="$router.push('/login')"
            target="_blank"
          >
            <span class="mr-2">Вход</span>
          </v-btn>
          <v-btn
            class="ma-2"
            text
            @click="$router.push('/register')"
            target="_blank"
          >
            <span class="mr-2">Регистрация</span>
          </v-btn>
        </span>
        <span v-else>
          <v-btn text @click="logout" target="_blank">
            <span class="mr-2">Выход</span>
          </v-btn>
        </span>
      </span>
    </v-app-bar>
  </v-container>
</template>

<script>
export default {
  name: "AppBar",
  data: function() {
    return {
      drawer: false
    };
  },
  methods: {
    logout: async function() {
      await this.$axios.post("/user/auth/logout");
      this.$store.commit("user/logout");
      await this.$router.push("/login");
    }
  }
};
</script>

<style scoped>
#title {
  cursor: pointer;
}
</style>
