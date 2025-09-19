<script lang="ts">
  import axios from "axios";
  import {showToastOnError} from "../../util/errorParser";
  import {goto} from "$app/navigation";

  let account = '';
  let password = '';
  let isLoading = false;

  const onClickLogin = showToastOnError(async () => {
    isLoading = true;
    await axios.post('/auth', {account, password});
    goto('/main');
  }, () => isLoading = false);

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter') onClickLogin();
  }
</script>

<p class="subtitle">로그인</p>
<label for="account">아이디</label>
<input id="account" type="text" on:keydown={onKeyDown}
       placeholder="아이디를 입력하세요" bind:value={account}/>
<label for="password">비밀번호</label>
<input id="password" type="password" on:keydown={onKeyDown}
       placeholder="비밀번호를 입력하세요" bind:value={password}/>
<button class="login-button" on:click={onClickLogin} disabled={isLoading}>
  로그인
</button>

<style lang="scss">
</style>
