from typing import List, Optional

class Entity:
    def __init__(self, id: int):
        self.id = id
        self.is_active = True
        self.tags = []

    def save(self):
        print(f"Saving {self.id}")

    def delete(self):
        print(f"Deleting {self.id}")

class User(Entity):
    def __init__(self, id: int, username: str, email: str):
        super().__init__(id)
        self.username = username
        self.email = email
        self.roles: List[str] = ["user"]
        self.first_name, self.last_name = "", ""

    def get_full_name(self) -> str:
        return f"{self.first_name} {self.last_name}"

    def add_role(self, role: str):
        self.roles.append(role)

class Product(Entity):
    def __init__(self, id: int, name: str, price: float):
        super().__init__(id)
        self.name = name
        self.price = price
        self.stock = 0

    def update_stock(self, quantity: int):
        self.stock += quantity

class Order:
    def __init__(self, order_id: str, user: User):
        self.order_id = order_id
        self.user = user
        self.items: List[Product] = []
        self.total = 0.0

    def add_item(self, product: Product, qty: int):
        self.items.append(product)
        self.total += product.price * qty

    def checkout(self):
        print("Checking out...")
