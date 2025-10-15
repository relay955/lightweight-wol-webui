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

<SvelteToast options={svelteToastOptions} />
{@render children?.()}

<style>
  :global {
    :root {
      /* 컬러 변수 */
      --color-white: #ffffff;
      --color-off-white: #fafafa;
      --color-light-gray: #f5f5f5;
      --color-gray: #e8e8e8;
      --color-border: #e0e0e0;
      --color-dark-gray: #666666;
      --color-black: #1a1a1a;
      --color-accent: #0a0a0a;

      /* 트랜지션 */
      --transition-base: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    }
    body{
      margin: 0;
    }
  }

</style>