<template>
  <div class="column has-padding">
    <!-- Build options -->
    <div class="tile is-ancestor">
      <div class="tile is-parent is-vertical">
        <div class="tile is-child is-12 box clickable-box" v-for="Lpackage in $root.$data.config.packages" :key="Lpackage.name" :index="Lpackage.name" v-on:click.capture.stop="clicked_box(Lpackage)">
          <div class="ribbon" v-if="Lpackage.is_new"><span>New!</span></div>
          <label class="checkbox">
            <b-checkbox v-model="Lpackage.default">
              <span v-if="!Lpackage.installed">Install</span> {{ Lpackage.name }}
            </b-checkbox>
            <span v-if="Lpackage.installed"><i>(installed)</i></span>
          </label>
          <div>
            <img class="package-icon" :src="`${publicPath + Lpackage.icon}`"/>
            <p style="padding-top: 4px;" class="package-description">
              {{ Lpackage.description }}
            </p>
            <p class="package-description">
              {{ get_extended_description(Lpackage) }}
            </p>
          </div>
        </div>
        <div class="tile is-child is-6 box clickable-box" v-if="!$root.$data.metadata.preexisting_install"  v-on:click.capture.stop="installDesktopShortcut = !installDesktopShortcut">
          <h4>Install Options</h4>
          <b-checkbox v-model="installDesktopShortcut">
            Create Desktop Shortcut
          </b-checkbox>
        </div>
      </div>
    </div>

    <div class="subtitle is-6" v-if="!$root.$data.metadata.preexisting_install && advanced">

    </div>


    <div class="subtitle is-6" v-if="!$root.$data.metadata.preexisting_install && advanced">Install Location</div>
    <div class="field has-addons" v-if="!$root.$data.metadata.preexisting_install && advanced">
      <div class="control is-expanded">
        <input class="input" type="text" v-model="$root.$data.install_location"
               placeholder="Enter a install path here">
      </div>
      <div class="control">
        <a class="button is-dark" v-on:click="select_file">
          Select
        </a>
      </div>
    </div>

    <div class="is-right-floating is-bottom-floating">
      <div class="field is-grouped">
        <p class="control">
          <a class="button is-medium" v-if="!$root.$data.config.hide_advanced && !$root.$data.metadata.preexisting_install && !advanced"
             v-on:click="advanced = true">Advanced...</a>
        </p>
        <p class="control">
          <a class="button is-dark is-medium" v-if="!$root.$data.metadata.preexisting_install"
             v-on:click="install">Install</a>
        </p>
        <p class="control">
          <a class="button is-dark is-medium" v-if="$root.$data.metadata.preexisting_install"
             v-on:click="install">Modify</a>
        </p>
      </div>
    </div>

    <div class="field is-grouped is-left-floating is-bottom-floating">
      <p class="control">
        <a class="button is-medium" v-if="$root.$data.metadata.preexisting_install"
           v-on:click="go_back">Back</a>
      </p>
    </div>
  </div>
</template>

<script>
  export default {
    name: 'SelectPackages',
    created: function() {
      // If they are authorized, make the packages that require authorization default
      // and also deselect any packages that don't use authorization
      if (this.$root.$data.has_reward_tier) {
        for (let package_index = 0; package_index < this.$root.config.packages.length; package_index++) {
          let current_package = this.$root.config.packages[package_index];
          current_package.default = current_package.requires_authorization;
        }
      }
    },
    data: function () {
      return {
        publicPath: process.env.BASE_URL,
        advanced: false,
        installDesktopShortcut: true
      }
    },
    methods: {
      select_file: function () {
        window.external.invoke(JSON.stringify({
          SelectInstallDir: {
            callback_name: 'selectFileCallback'
          }
        }))
      },
      install: function () {
        this.$router.push('/install/regular/' + this.installDesktopShortcut.toString())
      },
      go_back: function () {
        this.$router.go(-1)
      },
      show_authentication: function () {
        this.$router.push('/authentication')
      },
      show_authorization: function () {
        this.$router.push('/authentication')
      },
      installable: function (pkg) {
        return !pkg.requires_authorization || (pkg.requires_authorization && this.$root.$data.has_reward_tier);
      },
      clicked_box: function (pkg) {
        if (this.installable(pkg)) {
          pkg.default = !pkg.default;
        } else if (pkg.requires_authorization && !this.$root.$data.is_authenticated) {
          this.show_authentication()
        } else if (pkg.requires_authorization && !this.$root.$data.is_linked) {
          this.show_authorization()
        } else if (pkg.requires_authorization && !this.$root.$data.is_subscribed) {
          this.show_authorization()
        } else { // need_reward_tier_description
          this.show_authorization()
        }
      },
      get_extended_description: function(pkg) {
        if (!pkg.extended_description) {
          return "";
        }
        if (this.installable(pkg)) {
          return pkg.extended_description.no_action_description;
        } else if (pkg.requires_authorization && !this.$root.$data.is_authenticated) {
          return pkg.extended_description.need_authentication_description;
        } else if (pkg.requires_authorization && !this.$root.$data.is_linked) {
          return pkg.extended_description.need_link_description;
        } else if (pkg.requires_authorization && !this.$root.$data.is_subscribed) {
          return pkg.extended_description.need_subscription_description;
        } else { // need_reward_tier_description
          return pkg.extended_description.need_reward_tier_description;
        }
      }
    }
  }
</script>
