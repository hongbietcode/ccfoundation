import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { Layout } from "./components/Layout";
import { ConfigSwitcherPage } from "./pages/ConfigSwitcherPage";
import { ConfigEditorPage } from "./pages/ConfigEditorPage";
import { StoreEditPage } from "./pages/StoreEditPage";

const router = createBrowserRouter([
  {
    path: "/",
    element: <Layout />,
    children: [
      {
        index: true,
        element: <ConfigSwitcherPage />,
      },
      {
        path: "config/editor",
        element: <ConfigEditorPage />,
      },
      {
        path: "stores/new",
        element: <StoreEditPage />,
      },
      {
        path: "stores/:storeName/edit",
        element: <StoreEditPage />,
      },
    ],
  },
]);

export function Router() {
  return <RouterProvider router={router} />;
}