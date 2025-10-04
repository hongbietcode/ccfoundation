import { useNavigate } from "react-router-dom";
import { useStores, useCurrentStore, useSetCurrentStore } from "../lib/query";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { StoreManager } from "../components/StoreManager";

export function ConfigSwitcherPage() {
  const navigate = useNavigate();
  const { data: stores } = useStores();
  const { data: currentStore } = useCurrentStore();
  const setCurrentStoreMutation = useSetCurrentStore();

  const handleSelectStore = (storeName: string) => {
    setCurrentStoreMutation.mutate(storeName);
  };

  const handleEditConfig = () => {
    navigate("/config/editor");
  };

  const handleCreateStore = () => {
    navigate("/stores/new");
  };

  const handleEditStore = (storeName: string) => {
    navigate(`/stores/${storeName}/edit`);
  };

  return (
    <div className="max-w-6xl mx-auto space-y-6">
      {currentStore && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              Current Store: {currentStore.name}
              <div className="flex gap-2">
                <Button variant="outline" onClick={handleEditConfig}>
                  Edit Config
                </Button>
                <Button
                  variant="outline"
                  onClick={() => handleEditStore(currentStore.name)}
                >
                  Edit Store
                </Button>
              </div>
            </CardTitle>
            <CardDescription>
              This is the currently active configuration store
            </CardDescription>
          </CardHeader>
        </Card>
      )}

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              Configuration Stores
              <Button onClick={handleCreateStore}>
                Create New Store
              </Button>
            </CardTitle>
            <CardDescription>
              Select a store to use as your active configuration
            </CardDescription>
          </CardHeader>
          <CardContent>
            {stores && stores.length > 0 ? (
              <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                {stores.map((store) => (
                  <Card
                    key={store.name}
                    className={`cursor-pointer transition-colors hover:bg-muted/50 ${
                      currentStore?.name === store.name ? 'ring-2 ring-primary' : ''
                    }`}
                    onClick={() => handleSelectStore(store.name)}
                  >
                    <CardHeader className="pb-3">
                      <CardTitle className="text-lg">{store.name}</CardTitle>
                      <CardDescription>
                        {currentStore?.name === store.name && (
                          <span className="text-primary font-medium">Currently Active</span>
                        )}
                      </CardDescription>
                    </CardHeader>
                    <CardContent>
                      <div className="text-sm text-muted-foreground space-y-1">
                        <div>Model: {store.settings?.model || 'Default'}</div>
                        {store.settings?.permissions && (
                          <div>Permissions: {Object.keys(store.settings.permissions).length} configured</div>
                        )}
                        {store.settings?.env && (
                          <div>Environment: {Object.keys(store.settings.env).length} variables</div>
                        )}
                      </div>
                      <div className="flex gap-2 mt-3">
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleEditStore(store.name);
                          }}
                        >
                          Edit
                        </Button>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleSelectStore(store.name);
                          }}
                          disabled={currentStore?.name === store.name}
                        >
                          {currentStore?.name === store.name ? 'Active' : 'Select'}
                        </Button>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            ) : (
              <div className="text-center py-8">
                <p className="text-muted-foreground mb-4">No configuration stores found</p>
                <Button onClick={handleCreateStore}>Create Your First Store</Button>
              </div>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Store Management</CardTitle>
            <CardDescription>
              Advanced store operations and management
            </CardDescription>
          </CardHeader>
          <CardContent>
            <StoreManager />
          </CardContent>
        </Card>

        {setCurrentStoreMutation.error && (
          <Card className="border-destructive">
            <CardContent className="pt-6">
              <p className="text-destructive text-sm">
                Error switching store: {setCurrentStoreMutation.error.message}
              </p>
            </CardContent>
          </Card>
        )}

        {setCurrentStoreMutation.isSuccess && (
          <Card className="border-green-600">
            <CardContent className="pt-6">
              <p className="text-green-600 text-sm">
                Store switched successfully!
              </p>
            </CardContent>
          </Card>
        )}
    </div>
  );
}