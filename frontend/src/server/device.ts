interface PostDeviceReq {
  id: number|null;
  name: string;
  mac: string;
  ip: string;
}

interface GetDeviceRes {
  id: number;
  name: string;
  mac: string;
  ip: string;
  order_num: number;
}

