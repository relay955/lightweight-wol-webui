<script lang="ts">
  import axios from "axios";
  import {showToastOnError} from "../../util/errorParser";
  import {goto} from "$app/navigation";
  import {onMount} from "svelte";

  let user_name = '';
  let password = '';
  let passwordConfirm = '';
  let isLoading = false;

  // 페이지 로드 시 이미 계정이 있는지 확인
  onMount(showToastOnError(async () => {
    const res = await axios.get('/check-first-user');
    if (res.data.has_user) {
      // 이미 계정이 있으면 로그인 페이지로 리다이렉트
      goto('/login');
    }
  }));

  const onClickCreate = showToastOnError(async () => {
    if (!user_name.trim()) {
      throw new Error('Username is required');
    }
    if (password.length < 8) {
      throw new Error('Password must be at least 8 characters');
    }
    if (password !== passwordConfirm) {
      throw new Error('Passwords do not match');
    }

    isLoading = true;
    await axios.post('/join', {user_name, password});
    goto('/login');
  }, () => isLoading = false);

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter') onClickCreate();
  }
</script>

<div class="auth-container">
  <div class="auth-card wide">
    <div class="icon-wrapper">
      <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/>
        <circle cx="9" cy="7" r="4"/>
        <line x1="19" y1="8" x2="19" y2="14"/>
        <line x1="22" y1="11" x2="16" y2="11"/>
      </svg>
    </div>

    <h1 class="page-title">Initial Account Setup</h1>
    <p class="page-subtitle">Create an administrator account to get started</p>

    <form class="common-form" on:submit|preventDefault={onClickCreate}>
      <div class="form-group">
        <label for="username">Username</label>
        <input 
          id="username" 
          type="text" 
          on:keydown={onKeyDown}
          placeholder="Enter administrator username" 
          bind:value={user_name}
          disabled={isLoading}
          required
        />
      </div>

      <div class="form-group">
        <label for="password">Password</label>
        <input 
          id="password" 
          type="password" 
          on:keydown={onKeyDown}
          placeholder="Enter password (at least 8 characters)" 
          bind:value={password}
          disabled={isLoading}
          required
          minlength="8"
        />
      </div>

      <div class="form-group">
        <label for="password-confirm">Confirm Password</label>
        <input 
          id="password-confirm" 
          type="password" 
          on:keydown={onKeyDown}
          placeholder="Re-enter password" 
          bind:value={passwordConfirm}
          disabled={isLoading}
          required
        />
      </div>

      <button class="action-button" type="button" on:click={onClickCreate} disabled={isLoading}>
        {#if isLoading}
          <span class="spinner"></span>
          Creating...
        {:else}
          Create Administrator Account
        {/if}
      </button>
    </form>

    <div class="info-box">
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="16" x2="12" y2="12"/>
        <line x1="12" y1="8" x2="12.01" y2="8"/>
      </svg>
      <span>This will be the first administrator account for the system. Please set a secure password.</span>
    </div>
  </div>
</div>

<style lang="scss">
  @import '../../globalstyles/auth';
</style>
