import { ToastContainer } from "react-toastify";

const Toaster = () => {
  return (
    <ToastContainer
      position="bottom-left"
      autoClose={7000}
      closeOnClick
      limit={3}
      theme="dark"
    />
  );
};

export default Toaster;
