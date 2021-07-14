<template>
  <v-layout fluid fill-height align-center justify-center>
    <v-flex xs12 sm9 md9>
      <v-select
        class="mt-10"
        v-model="value"
        :items="items"
        chips
        label="Выбранные дни"
        multiple
        outlined
        @input="fetchEnrollees"
      />
      <v-data-table
        :headers="headers"
        :items="enrollees"
        :search="search"
        item-key="id"
        class="elevation-1"
      >
        <template v-slot:top>
          <v-toolbar flat>
            <v-toolbar-title>Список студентов</v-toolbar-title>
            <v-divider class="mx-4" inset vertical></v-divider>
            <v-spacer></v-spacer>

            <v-text-field
              v-model="search"
              append-icon="mdi-magnify"
              label="Поиск"
              single-line
              hide-details
            ></v-text-field>
            <v-dialog v-model="dialog" max-width="500px">
              <v-card>
                <v-card-title>
                  <span class="text-h5">{{ "Редактирование" }}</span>
                </v-card-title>

                <v-card-text>
                  <v-container>
                    <v-row>
                      <v-col cols="12" sm="6" md="4">
                        <v-text-field
                          v-model="editedItem.lastName"
                          label="Фамилия"
                        ></v-text-field>
                      </v-col>
                      <v-col cols="12" sm="6" md="4">
                        <v-text-field
                          v-model="editedItem.name"
                          label="Имя"
                        ></v-text-field>
                      </v-col>
                      <v-col cols="12" sm="6" md="4">
                        <v-text-field
                          v-model="editedItem.patronymic"
                          label="Отчество"
                        ></v-text-field>
                      </v-col>
                      <v-col cols="12" sm="6" md="4">
                        <v-text-field
                          v-model="editedItem.username"
                          label="Тег"
                        ></v-text-field>
                      </v-col>
                      <v-col cols="12" sm="6" md="4">
                        <v-text-field
                          v-model="editedItem.phoneNumber"
                          label="Телефон"
                        ></v-text-field>
                      </v-col>
                      <v-col cols="12" sm="6" md="4">
                        <v-text-field
                          v-model="editedItem.date"
                          label="Дата"
                        ></v-text-field>
                      </v-col>
                      <v-col cols="12" sm="6" md="4">
                        <v-text-field
                          v-model="editedItem.time"
                          label="Время"
                        ></v-text-field>
                      </v-col>
                      <v-col cols="12" sm="6" md="4">
                        <v-checkbox
                          v-model="editedItem.processed"
                          label="Статус"
                        ></v-checkbox>
                      </v-col>
                    </v-row>
                  </v-container>
                </v-card-text>

                <v-card-actions>
                  <v-spacer></v-spacer>
                  <v-btn color="blue darken-1" text @click="close">
                    Отменить
                  </v-btn>
                  <v-btn color="blue darken-1" text @click="save">
                    Сохранить
                  </v-btn>
                </v-card-actions>
              </v-card>
            </v-dialog>
          </v-toolbar>
        </template>
        <template v-slot:item.processed="{ item }">
          <v-checkbox v-model="item.processed" @click="changeStatus(item)"/>
        </template>
        <template v-slot:item.actions="{ item }">
          <v-icon small class="mr-2" @click="editItem(item)">
            mdi-pencil
          </v-icon>
        </template>
      </v-data-table>
    </v-flex>
  </v-layout>
</template>

<script>
export default {
  data: () => ({
    items: [],
    value: [],
    search: "",
    dialog: false,
    timer: "",
    headers: [
      { text: "Фамилия", value: "lastName" },
      { text: "Имя", value: "name" },
      { text: "Отчество", value: "patronymic" },
      { text: "Дата", value: "date" },
      { text: "Время", value: "time" },
      { text: "Статус", value: "processed" },
      { text: "Тег", value: "username" },
      { text: "Телефон", value: "phoneNumber" },
      { text: "Взаимодействие", value: "actions", sortable: false }
    ],
    enrollees: [],
    editedItem: {
      lastName: "",
      name: "",
      patronymic: "",
      date: "",
      time: "",
      username: "",
      phoneNumber: "",
      processed: false
    },
    defaultItem: {
      lastName: "",
      name: "",
      patronymic: "",
      date: "",
      time: "",
      username: "",
      phoneNumber: "",
      processed: false
    }
  }),
  watch: {
    dialog(val) {
      val || this.close();
    }
  },
  created() {
    this.fetchDates().then(dates => {
      this.items = dates;
      const today = new Date();
      this.value = [
        // eslint-disable-next-line no-unused-vars
        dates.find(function(item, index, array) {
          return new Date(item) >= today;
        })
      ];
      this.fetchEnrollees();
    });
    this.timer = setInterval(() => {
      this.fetchEnrollees();
    }, 5000);
  },
  methods: {
    fetchDates: async function() {
      let response = await this.$axios.get("/admin/queue/dates");
      return response.data.dates;
    },
    fetchEnrollees: async function() {
      let response = await this.$axios.post(
        "/admin/queue/enrollees",
        this.value
      );
      this.enrollees = response.data.enrollees;
    },
    changeStatus: async function(item) {
      try {
        await this.$axios.post(
          `/admin/queue/processed/${item.id}/${item.processed}`
        );
      } catch (error) {
        if (error.response.status === 400) {
          this.$store.commit("message/error", error.response.data.message);
        }
      }
    },
    cancelAutoUpdate() {
      clearInterval(this.timer);
    },
    editItem(item) {
      this.editedIndex = this.enrollees.indexOf(item);
      this.editedItem = Object.assign({}, item);
      this.dialog = true;
    },
    close() {
      this.dialog = false;
      this.$nextTick(() => {
        this.editedItem = Object.assign({}, this.defaultItem);
        this.editedIndex = -1;
      });
    },
    save: async function() {
      if (this.editedIndex > -1) {
        Object.assign(this.enrollees[this.editedIndex], this.editedItem);
      } else {
        this.enrollees.push(this.editedItem);
      }
      try {
        await this.$axios.post("/admin/queue/update", this.editedItem);
      } catch (error) {
        if (error.response.status === 400) {
          this.$store.commit("message/error", error.response.data.message);
        }
      }
      this.close();
    }
  },
  beforeDestroy() {
    this.cancelAutoUpdate();
  }
};
</script>

<style scoped></style>
