<template>
  <v-layout fluid fill-height align-center justify-center>
    <v-flex xs12 sm8 md4>
      <v-card>
        <v-form ref="form" v-model="valid" @submit.prevent="submit()">
          <v-toolbar class="elevation-12" color="primary" dark flat>
            <v-toolbar-title>Регистрация</v-toolbar-title>
          </v-toolbar>
          <v-card-text>
            <v-text-field
              v-model="lastName"
              required
              maxlength="32"
              counter
              :rules="rules.name"
              label="Фамилия"
              name="lastName"
              type="text"
            ></v-text-field>
            <v-text-field
              v-model="name"
              required
              maxlength="32"
              counter
              :rules="rules.name"
              label="Имя"
              name="name"
              type="text"
            ></v-text-field>
            <v-text-field
              v-model="patronymic"
              required
              maxlength="32"
              counter
              :rules="rules.name"
              label="Отчество"
              name="patronymic"
              type="text"
            ></v-text-field>
            <v-text-field
              v-model="phoneNumber"
              required
              counter
              :rules="rules.phoneNumber"
              label="Номер телефона"
              name="phoneNumber"
              type="text"
            ></v-text-field>
            <v-menu
              v-model="dateMenu"
              :close-on-content-click="false"
              :nudge-right="40"
              transition="scale-transition"
              offset-y
              min-width="auto"
            >
              <template v-slot:activator="{ on, attrs }">
                <v-text-field
                  v-model="date"
                  label="Дата"
                  prepend-icon="mdi-calendar"
                  v-bind="attrs"
                  v-on="on"
                  :rules="rules.date"
                ></v-text-field>
              </template>
              <v-date-picker
                v-model="date"
                v-on:change="fetchRelevantTime"
                @input="dateMenu = false"
                no-title
                locale="ru"
                class="mt-4"
                :allowed-dates="allowedDates"
                min="2021-07-28"
                max="2021-08-12"
              ></v-date-picker>
            </v-menu>

            <v-switch
              v-model="exactTime"
              label="Ручной ввод времени"
              hide-details
            ></v-switch>
            <v-menu
              v-model="timeMenu"
              :close-on-content-click="false"
              :nudge-right="40"
              transition="scale-transition"
              offset-y
              min-width="auto"
            >
              <template v-slot:activator="{ on, attrs }">
                <v-text-field
                  v-model="time"
                  label="Время"
                  :readonly="getExactState()"
                  prepend-icon="mdi-clock"
                  v-bind="attrs"
                  v-on="on"
                  :rules="rules.time"
                ></v-text-field>
              </template>
              <v-time-picker
                v-on:click:hour="updateCurrentHour"
                v-model="time"
                @input="timeMenu = false"
                scrollable
                :landscape="$vuetify.breakpoint.smAndUp"
                format="24hr"
                :allowed-hours="allowedHours"
                :allowed-minutes="allowedMinutes"
                min="9:00"
                max="18:00"
                :disabled="!date"
              ></v-time-picker>
            </v-menu>
          </v-card-text>
          <v-card-actions>
            <v-btn :disabled="!valid" type="submit" color="success">
              Зарегистрироваться
            </v-btn>
          </v-card-actions>
        </v-form>
      </v-card>
    </v-flex>
  </v-layout>
</template>

<script>
export default {
  name: "Register",
  data: function() {
    return {
      lastName: "",
      name: "",
      patronymic: "",
      phoneNumber: "",
      date: "",
      time: "",
      currentHour: "",
      relevantTime: new Map([["", []]]),
      dateMenu: false,
      timeMenu: false,
      exactTime: false,
      valid: true,
      rules: {
        name: [
          v => v.length <= 32 || "Максимум 32 символа",
          v => v.length >= 3 || "Минимум 3 символа",
          v => !!v || "Имя обязательно",
          v => /^[А-ЯҐЄІЇЎ][а-яґєії']+$/.test(v) || "Некоректное имя"
        ],
        phoneNumber: [
          v => !!v || "Номер обязателен",
          v => /^\+?3?8?(0\d{9})$/.test(v) || "Некоректный формат номера"
        ],
        date: [v => !!v || "Дата обязательна"],
        time: [v => !!v || "Время обязательно"]
      }
    };
  },
  methods: {
    submit: async function() {
      try {
        let response = await this.$axios.post("admin/queue/register", {
          last_name: this.lastName,
          name: this.name,
          patronymic: this.patronymic,
          phone_number: this.phoneNumber,
          date: this.date,
          time: this.time
        });
        this.$store.commit(
          "message/ok",
          "Порядковый номер для вызова: " + response.data.id
        );
        await this.fetchRelevantTime();
      } catch (error) {
        if (error.response.status === 400) {
          this.$store.commit("message/error", error.response.data.message);
        }
      }
    },
    getExactState: function() {
      return !this.exactTime;
    },
    fetchRelevantTime: async function() {
      try {
        let response = await this.$axios.post(
          `admin/queue/relevant-time/${this.date}`
        );
        this.relevantTime = new Map(Object.entries(response.data.relevantTime));
      } catch (error) {
        if (error.response.status === 400) {
          this.$store.commit("message/error", error.response.data.message);
        }
      }
    },
    updateCurrentHour: function(newHour) {
      this.currentHour = newHour;
    },
    allowedDates: val => Date.parse(val) >= new Date(new Date().toDateString()),
    allowedHours: function(v) {
      if (!this.exactTime) {
        return this.relevantTime.has(v.toString());
      }
      return true;
    },
    allowedMinutes: function(v) {
      if (!this.exactTime) {
        return this.relevantTime.get(this.currentHour.toString()).includes(v);
      }
      return true;
    }
  }
};
</script>

<style scoped></style>
