<script lang="ts">
  import axios from "axios";
  import {showToastOnError} from "../../util/errorParser";
  import {goto} from "$app/navigation";

  let user_name = '';
  let password = '';
  let isLoading = false;

  const onClickLogin = showToastOnError(async () => {
    isLoading = true;
    await axios.post('/login', {user_name, password});
    goto('/main');
  }, () => isLoading = false);

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter') onClickLogin();
  }
</script>

<div class="login-container">
  <div class="login-card">
    <h1 class="title">Lightweight WOL WebUI</h1>
    <p class="subtitle">Authentication</p>

    <form class="login-form" on:submit|preventDefault={onClickLogin}>
      <div class="form-group">
        <label for="account">Username</label>
        <input 
          id="account" 
          type="text" 
          on:keydown={onKeyDown}
          placeholder="Enter your username" 
          bind:value={user_name}
          disabled={isLoading}
        />
      </div>

      <div class="form-group">
        <label for="password">Password</label>
        <input 
          id="password" 
          type="password" 
          on:keydown={onKeyDown}
          placeholder="Enter your password" 
          bind:value={password}
          disabled={isLoading}
        />
      </div>

      <button class="login-button" type="button" on:click={onClickLogin} disabled={isLoading}>
        {#if isLoading}
          <span class="spinner"></span>
          Logging in...
        {:else}
          Login
        {/if}
      </button>
    </form>
  </div>
</div>

<style lang="scss">
  .login-container {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    background: linear-gradient(135deg, #fafafa 0%, #f0f0f0 100%);
    position: relative;
    overflow: hidden;

    // 배경 패턴
    &::before {
      content: '';
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background-image: 
        repeating-linear-gradient(0deg,
            transparent, transparent 2px,
            rgba(0, 0, 0, 0.015) 2px,
            rgba(0, 0, 0, 0.015) 4px
        );
      pointer-events: none;
    }
  }

  .login-card {
    background: var(--color-white);
    border-radius: 16px;
    padding: 48px 40px;
    width: 100%;
    max-width: 440px;
    box-shadow: 
      0 8px 24px rgba(0, 0, 0, 0.06),
      0 16px 48px rgba(0, 0, 0, 0.08);
    position: relative;
    border: 1px solid rgba(0, 0, 0, 0.06);

    // 상단 강조선
    &::before {
      content: '';
      position: absolute;
      top: 0;
      left: 50%;
      transform: translateX(-50%);
      width: 60px;
      height: 4px;
      background: var(--color-black);
      border-radius: 0 0 4px 4px;
    }
  }

  .title {
    font-size: 27px;
    font-weight: bold;
    color: var(--color-black);
    margin: 0 0 8px 0;
    text-align: center;
  }

  .subtitle {
    font-size: 15px;
    color: var(--color-dark-gray);
    margin: 0 0 40px 0;
    text-align: center;
    font-weight: normal;
  }

  .login-form {
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

  .login-button {
    margin-top: 16px;
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

    .spinner {
      width: 16px;
      height: 16px;
      border: 2px solid var(--color-white);
      border-top-color: transparent;
      border-radius: 50%;
      animation: spin 0.8s linear infinite;
    }

    @keyframes spin {
      to {
        transform: rotate(360deg);
      }
    }
  }

  // 반응형 디자인
  @media (max-width: 480px) {
    .login-container {
      padding: 16px;
    }

    .login-card {
      padding: 40px 24px;
    }

    .title {
      font-size: 28px;
    }

    .subtitle {
      font-size: 14px;
    }
  }
</style>
