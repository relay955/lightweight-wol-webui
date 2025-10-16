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

<style lang="scss">
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

    body {
      margin: 0;
    }

    /* 공통 Form 스타일 */
    .common-form {
      display: flex;
      flex-direction: column;
      gap: 24px;
    }

    .form-group {
      display: flex;
      flex-direction: column;
      gap: 10px;

      label {
        font-size: 14px;
        font-weight: normal;
        color: var(--color-black);
        letter-spacing: 0.3px;
        text-transform: uppercase;
        margin-left: 2px;
      }

      input {
        width: 100%;
        padding: 10px 12px;
        font-size: 16px;
        color: var(--color-black);
        background: var(--color-light-gray);
        border: 2px solid var(--color-border);
        border-radius: 8px;
        outline: none;
        transition: var(--transition-base);
        font-family: inherit;
        box-sizing: border-box;

        &::placeholder {
          color: rgba(0, 0, 0, 0.35);
        }

        &:focus {
          border-color: var(--color-black);
          background: var(--color-white);
          box-shadow: 0 0 0 4px rgba(0, 0, 0, 0.05);
        }

        &:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        &:hover:not(:disabled):not(:focus) {
          border-color: rgba(0, 0, 0, 0.3);
          background: var(--color-white);
        }
      }
    }

    /* 공통 액션 버튼 스타일 */
    .action-button {
      padding: 10px 16px;
      font-size: 16px;
      font-weight: normal;
      color: var(--color-white);
      background: var(--color-black);
      border: none;
      border-radius: 10px;
      cursor: pointer;
      transition: var(--transition-base);
      text-transform: uppercase;
      letter-spacing: 1px;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 12px;
      position: relative;
      overflow: hidden;
      
      &.small{
        padding: 6px 10px;
        font-size: 14px;
        border-radius: 6px;
      }

      &::before {
        content: '';
        position: absolute;
        top: 50%;
        left: 50%;
        width: 0;
        height: 0;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.1);
        transform: translate(-50%, -50%);
        transition: width 0.6s, height 0.6s;
      }

      &:hover:not(:disabled) {
        background: #2a2a2a;

        &::before {
          width: 300px;
          height: 300px;
        }
      }

      &:active:not(:disabled) {
        transform: translateY(0);
      }

      &:disabled {
        opacity: 0.7;
        cursor: not-allowed;
        transform: none;
      }
    }

    /* 공통 스피너 스타일 */
    .spinner {
      width: 16px;
      height: 16px;
      border: 2px solid var(--color-white);
      border-top-color: transparent;
      border-radius: 50%;
      animation: spin 0.8s linear infinite;

      &.large {
        width: 40px;
        height: 40px;
        border-width: 3px;
        border-color: rgba(0, 0, 0, 0.1);
        border-top-color: var(--color-black);
      }
    }

    @keyframes spin {
      to {
        transform: rotate(360deg);
      }
    }

    /* 체킹 컨테이너 */
    .checking-container {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 20px;
      padding: 40px 0;

      .checking-text {
        font-size: 15px;
        color: var(--color-dark-gray);
        margin: 0;
      }
    }

    /* 공통 Info Box 스타일 */
    .info-box {
      margin-top: 24px;
      padding: 16px;
      background: rgba(0, 0, 0, 0.03);
      border-radius: 8px;
      display: flex;
      align-items: flex-start;
      gap: 12px;
      font-size: 13px;
      color: var(--color-dark-gray);
      line-height: 1.6;
      border: 1px solid rgba(0, 0, 0, 0.06);

      svg {
        flex-shrink: 0;
        margin-top: 2px;
        color: var(--color-black);
      }
    }

    /* 반응형 디자인 */
    @media (max-width: 480px) {
      .common-form {
        gap: 20px;
      }
    }
  }
</style>