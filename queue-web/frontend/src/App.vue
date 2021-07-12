<template>
  <v-app>
    <v-dialog
      v-model="$store.getters['message/hasMessage']"
      persistent
      max-width="290"
    >
      <v-card>
        <v-card-title>{{
          $store.state.message.error ? "Ошибка" : "Сообщение"
        }}</v-card-title>
        <v-card-text>{{ $store.state.message.text }}</v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn
            :color="$store.getters['message/messageColor']"
            text
            @click="$store.commit('message/close')"
            >Понятно</v-btn
          >
        </v-card-actions>
      </v-card>
    </v-dialog>
    <app-bar />
    <v-main class="pt-10">
      <router-view class="background" />
    </v-main>
  </v-app>
</template>

<script>
import AppBar from "./components/AppBar";
export default {
  name: "App",
  components: { AppBar },
  data: () => ({
    //
  })
};
</script>

<style>
.background {
  background: url("assets/background.jpg") center;
  background-size: cover;
}
</style>
