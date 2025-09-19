import Cookies from "universal-cookie";
import {jwtDecode} from "jwt-decode";

export interface JWT {
  sub: string,
}

export const parseJWT = (): JWT => {
  try {
    let token = new Cookies().get("accessToken");
    return jwtDecode<JWT>(token);
  } catch (e) {
    return {sub: ""}
  }
}

export function checkIsAuthenticated() {
  const accessToken = parseJWT();
  return accessToken.sub !== "";
}
