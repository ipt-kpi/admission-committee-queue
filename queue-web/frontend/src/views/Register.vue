<template>
  <v-layout fluid fill-height align-center justify-center>
    <v-flex xs12 sm8 md4>
      <v-card>
        <v-form ref="form" v-model="valid" @submit.prevent="submit()">
          <v-toolbar class="elevation-12" color="primary" dark flat>
            <v-toolbar-title>Регистрация</v-toolbar-title>
          </v-toolbar>
          <div v-if="loaded">
            <v-card-text>
              <div v-if="!verify">
                <v-text-field
                  v-model="username"
                  required
                  maxlength="16"
                  counter
                  :rules="rules.username"
                  label="Логин"
                  name="login"
                  type="text"
                >
                </v-text-field>
                <v-text-field
                  v-model="email"
                  required
                  maxlength="32"
                  counter
                  :rules="rules.email"
                  label="Почта"
                  name="email"
                  type="email"
                >
                </v-text-field>
                <v-text-field
                  v-model="password"
                  :rules="rules.password"
                  maxlenght="32"
                  required
                  label="Пароль"
                  type="password"
                >
                </v-text-field>
                <v-text-field
                  v-model="confirmPassword"
                  required
                  :rules="confirmRules"
                  label="Повторите пароль"
                  type="password"
                >
                </v-text-field>
                <vue-recaptcha
                  ref="recaptcha"
                  @verify="register"
                  @expired="onCaptchaExpired"
                  sitekey="6Leb1rsZAAAAAICrfCBs9-ei4SZHWmKwgwMMta5T"
                  :loadRecaptchaScript="true"
                />
              </div>
              <div v-else>
                <p class="text-center green--text text--lighten-1">
                  Подтвердите свой аккаунт по почте.
                </p>
              </div>
            </v-card-text>
            <v-card-actions>
              <div v-if="!verify">
                <v-btn
                  :disabled="!valid || !captchaToken"
                  type="submit"
                  color="success"
                  >Зарегистрироваться</v-btn
                >
              </div>
            </v-card-actions>
          </div>
          <div v-else>
            <v-skeleton-loader type="article" />
          </div>
        </v-form>
      </v-card>
    </v-flex>
  </v-layout>
</template>

<script>
import VueRecaptcha from "vue-recaptcha";
export default {
  name: "Register",
  components: { VueRecaptcha },
  data: function() {
    return {
      loaded: true,
      username: "",
      password: "",
      confirmPassword: "",
      email: "",
      captchaToken: undefined,
      verify: false,
      valid: true,
      rules: {
        username: [
          v => v.length <= 16 || "Максимум 16 символов",
          v => v.length >= 3 || "Минимум 3 символа",
          v => !!v || "Имя обязательно",
          v => /^[a-zA-Z0-9_]+$/.test(v) || "Некоректное имя"
        ],
        email: [
          v => v.length <= 64 || "Максимум 64 символа",
          v => v.length >= 3 || "Минимум 3 символа",
          v => !!v || "Почта обязательна",
          v =>
            /^\w+([.-]?\w+)*@\w+([.-]?\w+)*(\.\w{2,3})+$/.test(v) ||
            "Некоректная почта"
        ],
        password: [
          v => v.length <= 32 || "Максимум 32 символа",
          v => v.length >= 6 || "Минимум 6 символа",
          v => !!v || "Пароль обязателен"
        ]
      }
    };
  },
  methods: {
    onCaptchaExpired: function() {
      this.$refs.recaptcha.reset();
    },
    register: function(token) {
      this.captchaToken = token;
    },
    submit: async function() {
      let recaptcha = this.$refs.recaptcha;
      this.loaded = false;
      try {
        await this.$axios.post("user/auth/register", {
          username: this.username,
          password: this.password,
          token: this.captchaToken,
          email: this.email
        });
        this.verify = true;
        this.loaded = true;
      } catch (error) {
        this.loaded = true;
        if (error.response.status === 400) {
          this.$store.commit("message/error", error.response.data.message);
        }
      }
      recaptcha.reset();
    }
  },
  computed: {
    confirmRules: function() {
      const rules = [];
      if (this.password) {
        const rule = v => (!!v && v) === this.password || "Пароли не совпадают";
        rules.push(rule);
      }
      return rules;
    }
  }
};
</script>

<style scoped></style>
