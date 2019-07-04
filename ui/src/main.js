import Vue from 'vue'
import App from './App.vue'
import router from './router'
import { ajax, stream_ajax } from './helpers'
import Buefy from 'buefy'
import 'buefy/dist/buefy.css'

Vue.config.productionTip = false
Vue.use(Buefy)

// Borrowed from http://tobyho.com/2012/07/27/taking-over-console-log/
function intercept (method) {
  console[method] = function () {
    var message = Array.prototype.slice.apply(arguments).join(' ')
    window.external.invoke(
      JSON.stringify({
        Log: {
          kind: method,
          msg: message
        }
      })
    )
  }
}

// See if we have access to the JSON interface
var has_external_interface = false;
try {
  window.external.invoke(JSON.stringify({
    Test: {}
  }))
  has_external_interface = true;
} catch (e) {
  console.warn("Running without JSON interface - unexpected behaviour may occur!")
}

// Overwrite loggers with the logging backend
if (has_external_interface) {
  window.onerror = function (msg, url, line) {
    window.external.invoke(
      JSON.stringify({
        Log: {
          kind: 'error',
          msg: msg + ' @ ' + url + ':' + line
        }
      })
    )
  }

  var methods = ['log', 'warn', 'error']
  for (var i = 0; i < methods.length; i++) {
    intercept(methods[i])
  }
}

// Disable F5
function disable_shortcuts (e) {
  switch (e.keyCode) {
    case 116: // F5
      e.preventDefault()
      break
  }
}

// Check to see if we need to enable dark mode
ajax('/api/dark-mode', function (enable) {
  if (enable) {
    document.body.classList.add('has-background-black-ter')
  }
})

window.addEventListener('keydown', disable_shortcuts)

document.getElementById('window-title').innerText =
  base_attributes.name + ' Installer'

function selectFileCallback (name) {
  app.install_location = name
}

var app = new Vue({
  router: router,
  data: {
    attrs: base_attributes,
    config: {},
    install_location: '',
    // If the option to pick an install location should be provided
    show_install_location: true,
    metadata: {
      database: [],
      install_path: '',
      preexisting_install: false
    }
  },
  render: function (caller) {
    return caller(App)
  },
  methods: {
    exit: function () {
      ajax(
        '/api/exit',
        function () {},
        function (msg) {
          var search_location = app.metadata.install_path.length > 0 ? app.metadata.install_path :
            "the location where this installer is";

          app.$router.replace({ name: 'showerr', params: { msg: msg +
                '\n\nPlease upload the log file (in ' + search_location + ') to ' +
                'the ' + app.attrs.name + ' team'
          }});
        }
      )
    },
    ajax: ajax,
    stream_ajax: stream_ajax
  }
}).$mount('#app')

console.log("Vue started")
