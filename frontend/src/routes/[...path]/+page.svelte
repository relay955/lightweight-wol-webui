<script>
  import {onMount} from "svelte";
  import {checkIsAuthenticated} from "../../util/loginManager.js";
  import {goto} from "$app/navigation";

  onMount(() => {
    goto(!checkIsAuthenticated() ? "/login" : "/main");
  })
</script>