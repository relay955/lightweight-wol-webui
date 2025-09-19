import {toast} from "@zerodevx/svelte-toast";

/**
 * 토스트로 한줄로 표현할 수 있는 에러메시지로 해석합니다.
 */
export const parseErrorMessageToOneLine = (e: any, fieldmap: { [index: string]: string } = {}): string => {
  try {
    if (typeof e === "string") return e;
    if (!e.response || !e.response.status) {
      console.log(e)
      if (e.message) return `에러가 발생했습니다. (${e.message})`
      else return "에러가 발생했습니다."
    }

    if (e.response.status === 403) return "권한이 없습니다."
    if (e.response.status === 500) return "에러가 발생했습니다. 잠시 후 다시 시도해주세요."

    const detail = e.response.data.detail
    if (!Array.isArray(detail)) return e.response.data.msg;

    if (detail.length == 0) return "검증에 실패했습니다.";
    return detail[0].msg;
  } catch (e) {
    console.log(e)
    return "에러가 발생했습니다."
  }
}

export type ValidateMessages = {
    field:string;
    message:string;
}[]

/** 예외처리를 편하게 할 수 있도록 하는 데코레이터입니다.<br/>
 * 실험적 기능을 사용하지 않으려는 의도와 예외처리를 간편하게 하고자 하는 의도를 모두 충족하도록 합니다.
 * 기술적 한계로, 인자가 없거나 3개까지의 함수만 중복 정의하였습니다. 인자 개수에 따라 사용하면 됩니다.
 * 또한, 서버 전송시 에러를 핸들링하는것을 기본적으로 전제하므로 async 함수를 리턴합니다.
 */
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

/** 예외 처리 데코레이터의 인자 1개 바리에이션. */
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

/** 예외 처리 데코레이터의 인자 2개 바리에이션. */
export const showToastOnErrorP2 = <T,T2>(fn:(arg1:T, arg2:T2)=>void) => {
  return async (arg1:T, arg2:T2) => {
    try {
      await fn(arg1,arg2);
    }catch (e){
      toast.push(parseErrorMessageToOneLine(e));
    }
  }
}

/** 예외 처리 데코레이터의 인자 3개 바리에이션. */
export const showToastOnErrorP3 = <T,T2,T3>(fn:(arg1:T, arg2:T2, arg3:T3)=>void) => {
  return async (arg1:T, arg2:T2, arg3:T3) => {
    try {
      await fn(arg1,arg2,arg3);
    }catch (e){
      toast.push(parseErrorMessageToOneLine(e));
    }
  }
}
