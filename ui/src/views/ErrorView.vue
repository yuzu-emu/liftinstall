<template>
    <div class="column has-padding">
            <h4 class="subtitle">An error occurred:</h4>

            <pre class="pre-wrap">{{ msg }}</pre>

            <div class="field is-grouped is-right-floating is-bottom-floating">
                <p class="control">
                    <a class="button is-primary is-medium" v-if="remaining && !is_launcher" v-on:click="go_back">Back</a>
                    <a class="button is-primary is-medium" v-if="is_launcher" v-on:click="exit">Exit</a>
                </p>
            </div>
    </div>
</template>

<style>
.pre-wrap {
  /* https://css-tricks.com/snippets/css/make-pre-text-wrap/ */
  white-space: pre-wrap;       /* css-3 */
  white-space: -moz-pre-wrap;  /* Mozilla, since 1999 */
  white-space: -pre-wrap;      /* Opera 4-6 */
  white-space: -o-pre-wrap;    /* Opera 7 */
  word-wrap: break-word;       /* Internet Explorer 5.5+ */
}
</style>

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
