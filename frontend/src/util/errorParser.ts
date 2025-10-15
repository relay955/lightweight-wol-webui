import {toast} from "@zerodevx/svelte-toast";

export const parseErrorMessageToOneLine = (e: any, fieldmap: { [index: string]: string } = {}): string => {
  try {
    if (typeof e === "string") return e;
    if (!e.response || !e.response.status) {
      console.log(e)
      if (e.message) return `An error occurred. (${e.message})`
      else return "An error occurred."
    }

    if (e.response.status === 403) return "Permission denied."
    if (e.response.status === 500) return "An error occurred. Please try again later."

    const detail = e.response.data.detail
    if (!Array.isArray(detail)) return e.response.data.msg;

    if (detail.length == 0) return "Validation failed.";
    return detail[0].msg;
  } catch (e) {
    console.log(e)
    return "An error occurred."
  }
}

export type ValidateMessages = {
    field:string;
    message:string;
}[]

export const showToastOnError = (fn:()=>void,onFinally?:()=>void) => {
  return async () => {
    try {
      await fn()
    }catch (e){
      toast.push(parseErrorMessageToOneLine(e));
    }finally {
      onFinally?.();
    }
  }
}

export const showToastOnErrorP1 = <T>(fn:(arg1:T)=>void,onFinally?:()=>void) => {
  return async (arg1:T) => {
    try {
      await fn(arg1)
    }catch (e){
      toast.push(parseErrorMessageToOneLine(e));
    }finally {
      onFinally?.();
    }
  }
}

export const showToastOnErrorP2 = <T,T2>(fn:(arg1:T, arg2:T2)=>void) => {
  return async (arg1:T, arg2:T2) => {
    try {
      await fn(arg1,arg2);
    }catch (e){
      toast.push(parseErrorMessageToOneLine(e));
    }
  }
}

export const showToastOnErrorP3 = <T,T2,T3>(fn:(arg1:T, arg2:T2, arg3:T3)=>void) => {
  return async (arg1:T, arg2:T2, arg3:T3) => {
    try {
      await fn(arg1,arg2,arg3);
    }catch (e){
      toast.push(parseErrorMessageToOneLine(e));
    }
  }
}
