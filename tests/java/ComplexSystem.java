import java.util.List;
import java.util.ArrayList;

interface Identifiable {
    String getId();
}

abstract class BaseEntity implements Identifiable {
    protected String id;

    public String getId() {
        return id;
    }
}

class User extends BaseEntity {
    private String username;
    private List<Role> roles;

    public User(String username) {
        this.username = username;
        this.roles = new ArrayList<>();
    }

    public void addRole(Role role) {
        this.roles.add(role);
    }
}

class Role {
    private String name;

    public Role(String name) {
        this.name = name;
    }
}

class Order {
    private User owner;
    private List<Product> products;

    public void setOwner(User user) {
        this.owner = user;
    }

    public Product processOrder(Product input) {
        return input;
    }
}

class Product {
    private String sku;
}
