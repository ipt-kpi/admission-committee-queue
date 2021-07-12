<template>
  <v-layout fluid fill-height align-center justify-center>
    <v-flex xs12 sm8 md6>
      <v-card>
        <v-form ref="form" v-model="valid" @submit.prevent="submit()">
          <v-toolbar dark class="elevation-12" color="primary" flat>
            <v-toolbar-title>Вход</v-toolbar-title>
          </v-toolbar>
          <v-card-text>
            <v-text-field
              v-model="name"
              required
              maxlength="16"
              counter
              :rules="rules.name"
              label="Логин"
              name="login"
              type="text"
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
          </v-card-text>
          <v-card-actions>
            <v-btn
              :disabled="!valid || !fingerprint"
              type="submit"
              color="success"
              >Войти</v-btn
            >
          </v-card-actions>
        </v-form>
      </v-card>
    </v-flex>
  </v-layout>
</template>

<script>
export default {
  name: "Login",
  data: function() {
    return {
      name: "",
      password: "",
      valid: true,
      fingerprint: undefined,
      rules: {
        name: [
          v => (v.length <= 16 && v.length >= 3) || "Максимум 16 символов"
        ],
        password: [
          v =>
            (v.length <= 32 && v.length >= 6) ||
            "Пароль должен быть от 6 символов"
        ]
      }
    };
  },
  created() {
    this.$fingerprint.get(components => {
      this.fingerprint = this.$fingerprint.x64hash128(
        components
          .map(pair => {
            return pair.value;
          })
          .join(),
        31
      );
    });
  },
  methods: {
    submit: async function() {
      try {
        let response = await this.$axios.post("/user/auth/login", {
          username: this.name,
          password: this.password,
          fingerprint: this.fingerprint
        });

        const auth = {
          data: response.data,
          login: this.name
        };
        if (response.status === 200) {
          this.$store.commit("user/login", auth);
          await this.$router.push("/");
        }
      } catch (error) {
        if (error.response.status === 400) {
          this.$store.commit("message/error", error.response.data.message);
        }
      }
    }
  }
};
</script>

<style scoped></style>
