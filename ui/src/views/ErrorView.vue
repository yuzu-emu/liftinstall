<template>
    <div class="column has-padding">
            <h4 class="subtitle">An error occurred:</h4>

            <pre>{{ msg }}</pre>

            <div class="field is-grouped is-right-floating is-bottom-floating">
                <p class="control">
                    <a class="button is-primary is-medium" v-if="remaining" v-on:click="go_back">Back</a>
                </p>
            </div>
    </div>
</template>

<script>
export default {
  name: 'ErrorView',
  data: function () {
    return {
      msg: this.$route.params.msg,
      remaining: window.history.length > 1
    }
  },
  methods: {
    go_back: function () {
      this.$router.go(-1)
    }
  }
}
</script>
