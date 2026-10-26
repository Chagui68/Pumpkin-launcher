package dev.lone.itemsadder.api;

// Compile-only stub for local test builds: the real API is no longer published.
public class CustomBlock {
    private static UnsupportedOperationException stub() { return new UnsupportedOperationException("ItemsAdder API stub"); }
    public static java.util.Set<String> getNamespacedIdsInRegistry() { throw stub(); }
    public static CustomBlock place(String namespacedId, org.bukkit.Location location) { throw stub(); }
    public static boolean remove(org.bukkit.Location location) { throw stub(); }
    public static boolean isInRegistry(String namespacedId) { throw stub(); }
    public static CustomBlock getInstance(String namespacedId) { throw stub(); }
    public org.bukkit.inventory.ItemStack getItemStack() { throw stub(); }
    public String getDisplayName() { throw stub(); }
    public static class Advanced {
        public static String getInCustomRegion(org.bukkit.Location location) { throw stub(); }
        public static void deleteAllCustomBlocksInChunk(org.bukkit.Chunk chunk) { throw stub(); }
        public static java.util.List<org.bukkit.Location> getAllBlocksLocationsList(org.bukkit.Chunk chunk) { throw stub(); }
        public static java.util.Map<org.bukkit.Location, String> getAllBlocksLocations(org.bukkit.Chunk chunk) { throw stub(); }
        public static void runActionOnBlocks(org.bukkit.Chunk chunk, java.util.function.BiConsumer<String, org.bukkit.Location> action) { throw stub(); }
    }
}
