<script lang="ts">
  import axios from "axios";
	import favicon from '$lib/assets/favicon.svg';
  import Cookies from "universal-cookie";
  import {SvelteToast, type SvelteToastOptions} from "@zerodevx/svelte-toast"
  import {envMode, requestURL} from "../config";

  let { children } = $props();

  axios.defaults.withCredentials = true;
  axios.defaults.baseURL = `${window.location.origin}/api`;
  if (envMode === "development") axios.defaults.baseURL = requestURL;
  axios.interceptors.response.use(
      res => res,
      async error => {
        if (error.response === undefined) return Promise.reject(error);
        const status = error.response.status;
        if (status === 401) {
          let cookie = new Cookies();
          cookie.remove("accessToken", {path: "/"});
          cookie.remove("refreshToken", {path: "/"});
          window.location.href = "/";
        }
        return Promise.reject(error);
      }
  )

  const svelteToastOptions:SvelteToastOptions = {
    dismissable:false
  }
  
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children?.()}
