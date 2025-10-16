interface PostDeviceReq {
  id: number|null;
  name: string;
  mac: string;
}

interface GetDeviceRes {
  id: number;
  name: string;
  mac: string;
  order_num: number;
}

